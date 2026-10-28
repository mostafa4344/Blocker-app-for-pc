// ContentGuard extension background: keeps blocking rules in sync with the local
// agent and reports which site is in use so daily limits can be counted.
if (typeof buildRules === "undefined") importScripts("rules.js");

const ext = globalThis.browser ?? chrome;
const AGENT = "http://127.0.0.1:17345";
const BROWSER = navigator.userAgent.includes("Firefox") ? "firefox" : "chromium";

/** Which browser product this is, so the desktop app can tell e.g. Chrome
 *  (protected) from Edge (no extension) and close only the unprotected one. */
async function detectBrand() {
  if (BROWSER === "firefox") return "firefox";
  if (navigator.brave && (await navigator.brave.isBrave?.().catch(() => false))) return "brave";
  const brands = (navigator.userAgentData?.brands ?? []).map((b) => b.brand);
  if (brands.includes("Microsoft Edge")) return "edge";
  if (brands.includes("Brave")) return "brave";
  if (brands.some((b) => b.startsWith("Opera"))) return "opera";
  if (brands.includes("YaBrowser") || /YaBrowser/.test(navigator.userAgent)) return "yandex";
  if (brands.includes("Google Chrome")) return "chrome";
  return "generic";
}
const BRAND = detectBrand();
const BLOCKED_PAGE = ext.runtime.getURL("blocked.html");
const HEARTBEAT_MINUTES = 0.5;

let lastHost = null;

async function agent(path, body) {
  const res = await fetch(`${AGENT}${path}`, body === undefined ? {} : {
    method: "POST",
    headers: { "Content-Type": "application/json", "X-ContentGuard": "1" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`agent ${res.status}`);
  return res.json();
}

async function syncPolicy() {
  let policy;
  try {
    policy = await agent(`/api/ext/policy?browser=${BROWSER}&brand=${await BRAND}`);
  } catch {
    // Agent unreachable: keep the last rules (blocks stay in force).
    await ext.storage.local.set({ agentOnline: false });
    return;
  }
  await ext.storage.local.set({ policy, agentOnline: true, policyAt: Date.now() });
  const rules = buildRules(policy, BLOCKED_PAGE);
  const existing = await ext.declarativeNetRequest.getDynamicRules();
  await ext.declarativeNetRequest.updateDynamicRules({ removeRuleIds: existing.map((r) => r.id), addRules: rules });
}

function hostOf(url) {
  try {
    return new URL(url).hostname;
  } catch {
    return "";
  }
}

const isWeb = (url) => /^https?:\/\//i.test(url || "");

/** Reports the focused tab; redirects it if the agent says it is blocked now. */
async function report(tab, seconds, opened) {
  if (!tab || !isWeb(tab.url)) return;
  try {
    const r = await agent("/api/ext/activity", { url: tab.url, seconds, opened, browser: BROWSER, brand: await BRAND });
    if (r.blocked) {
      await ext.tabs.update(tab.id, { url: `${BLOCKED_PAGE}#${tab.url}` });
      syncPolicy();
    }
  } catch {
    /* agent offline */
  }
}

async function focusedTab() {
  const win = await ext.windows.getLastFocused({ populate: false }).catch(() => null);
  if (!win || !win.focused) return null;
  const [tab] = await ext.tabs.query({ active: true, windowId: win.id });
  return tab ?? null;
}

async function heartbeat() {
  await syncPolicy();
  const state = await ext.idle.queryState(60);
  const tab = await focusedTab();
  if (state === "active" && tab) await report(tab, HEARTBEAT_MINUTES * 60, false);
}

// A newly opened site counts as an "open" for open-count limits.
async function onNavigate(tab) {
  if (!tab?.active || !isWeb(tab.url)) return;
  const host = hostOf(tab.url);
  const opened = host !== lastHost;
  lastHost = host;
  await report(tab, 0, opened);
}

ext.tabs.onUpdated.addListener((_id, info, tab) => info.url && onNavigate(tab));
ext.tabs.onActivated.addListener(async ({ tabId }) => onNavigate(await ext.tabs.get(tabId)));
ext.alarms.onAlarm.addListener((a) => a.name === "heartbeat" && heartbeat());
ext.runtime.onInstalled.addListener(() => syncPolicy());
ext.runtime.onStartup.addListener(() => syncPolicy());
ext.alarms.create("heartbeat", { periodInMinutes: HEARTBEAT_MINUTES });
syncPolicy();
