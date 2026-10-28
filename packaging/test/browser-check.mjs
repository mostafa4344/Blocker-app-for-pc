// Real-browser check for HTL Content Guard PC (Windows test PC).
// Opens a Chromium browser with a fresh, throw-away profile and remote
// debugging, visits each URL and reports whether it loaded or was blocked.
// Machine-wide browser policies (force-installed extension, private-window
// policy) still apply to the fresh profile, so this tests the real setup.
//
//   node packaging/test/browser-check.mjs <browser.exe> <url> [url...]
//   node packaging/test/browser-check.mjs <browser.exe> --incognito-check
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const [exe, ...args] = process.argv.slice(2);
if (!exe) {
  console.error("usage: browser-check.mjs <browser.exe> <url> [url...]");
  process.exit(2);
}
const port = 9300 + Math.floor(Math.random() * 600);
const profile = mkdtempSync(path.join(tmpdir(), "htl-check-"));
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

const child = spawn(exe, [
  `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`,
  "--no-first-run", "--no-default-browser-check", "--disable-sync", "--window-size=1100,750", "about:blank",
], { stdio: "ignore", detached: false });

async function json(p) {
  for (let i = 0; i < 60; i++) {
    try { return await (await fetch(`http://127.0.0.1:${port}${p}`)).json(); } catch { await wait(500); }
  }
  throw new Error("browser did not open its debugging port");
}

function cdp(wsUrl) {
  const ws = new WebSocket(wsUrl);
  let id = 0;
  const pending = new Map();
  const events = [];
  ws.onmessage = (m) => {
    const d = JSON.parse(m.data);
    if (d.id && pending.has(d.id)) { pending.get(d.id)(d); pending.delete(d.id); } else events.push(d);
  };
  const ready = new Promise((r) => (ws.onopen = r));
  // Every call gives up after 8 s, so a tab the extension redirected can't hang the run.
  const send = (method, params = {}) => new Promise((r) => {
    const i = ++id;
    pending.set(i, r);
    ws.send(JSON.stringify({ id: i, method, params }));
    setTimeout(() => pending.has(i) && (pending.delete(i), r({ error: { message: "timeout" } })), 8000);
  });
  return { ready, send, events, close: () => ws.close() };
}

// Each URL opens in a new tab: the extension may swap the tab for its block page.
async function visit(browser, url) {
  const created = await browser.send("Target.createTarget", { url: "about:blank" });
  const targetId = created.result?.targetId;
  let t = (await json("/json/list")).find((x) => x.id === targetId);
  const page = cdp(t.webSocketDebuggerUrl);
  await page.ready;
  const nav = await page.send("Page.navigate", { url });
  await wait(9000);
  const r = await page.send("Runtime.evaluate", {
    expression: "JSON.stringify({u: location.href, t: document.title, n: (document.body && document.body.innerText || '').length, e: document.documentElement.innerHTML.match(/ERR_[A-Z_]+/)?.[0] || ''})",
    returnByValue: true,
  });
  page.close();
  t = (await json("/json/list")).find((x) => x.id === targetId) || {};
  const v = r.result?.result?.value ? JSON.parse(r.result.result.value) : { u: t.url, t: t.title };
  const err = nav.result?.errorText || v.e || "";
  let verdict;
  const finalUrl = v.u || t.url || "";
  if (finalUrl.startsWith("chrome-extension://") || /blocked\.html/.test(finalUrl)) verdict = "BLOCKED by extension (block page)";
  else if (err) verdict = `BLOCKED (${err})`;
  else if ((v.n || 0) > 200) verdict = "LOADED";
  else verdict = `unclear (text ${v.n})`;
  console.log(`${url.padEnd(40)} ${verdict.padEnd(38)} title="${(v.t || "").slice(0, 40)}"`);
}

try {
  const version = await json("/json/version");
  console.log(`browser: ${version.Browser}`);
  await wait(Number(process.env.EXT_WAIT || 15000)); // let force-installed extensions download and load
  const targets = await json("/json/list");
  const exts = targets.filter((t) => t.url.startsWith("chrome-extension://")).map((t) => t.url.split("/")[2]);
  console.log(`extensions running: ${[...new Set(exts)].join(", ") || "none"}`);
  const b = cdp(version.webSocketDebuggerUrl);
  await b.ready;
  if (args[0] === "--incognito-check") {
    const r = await b.send("Target.createBrowserContext", {});
    const c = r.result?.browserContextId ? await b.send("Target.createTarget", { url: "about:blank", browserContextId: r.result.browserContextId }) : r;
    console.log(`private window: ${c.error ? "REFUSED (" + c.error.message + ")" : "allowed"}`);
  } else {
    for (const u of args) await visit(b, u);
  }
} catch (e) {
  console.log(`ERROR ${e.message}`);
} finally {
  try { child.kill(); } catch {}
  process.exit(0);
}
