// ContentGuard UI entry point: navigation, rendering, onboarding and polling.
import { S, $, $$, t, presetName, addPreset, num, esc, languageOptions, getLanguage, api, cfg, clone, refresh, openModal, closeModal, modalOpen, updateCountdowns, toast, errMsg } from "/core.js";
import { renderHome } from "/home.js";
import { renderLists, renderSchedules, renderLimits } from "/editors.js";
import { renderProtection, renderSettings, renderLicense } from "/protection.js";
import { initTips } from "/help.js";

initTips();

const PAGES = [
  ["home", "🏠", "nav_home", renderHome],
  ["lists", "🚫", "nav_lists", renderLists],
  ["schedules", "🗓️", "nav_schedules", renderSchedules],
  ["limits", "⏳", "nav_limits", renderLimits],
  ["protection", "🛡️", "nav_protection", renderProtection],
  ["premium", "⭐", "nav_premium", renderLicense],
  ["settings", "⚙️", "nav_settings", renderSettings],
];

/** A persistent banner for trial countdown / expiry, shown on every page. */
function licenseBanner() {
  const lic = S.status.license;
  const st = lic.status.state;
  if (st === "premium") return "";
  const to = `<button class="btn small" data-page="premium">${t("get_premium")}</button>`;
  if (st === "trial") {
    return `<div class="banner info row between"><span>${t("trial_days_left", { days: num(lic.status.days_left) })}</span>${to}</div>`;
  }
  if (st === "expired") return `<div class="banner warn row between"><span>${t("trial_expired")}</span>${to}</div>`;
  if (st === "locked_offline") return `<div class="banner danger row between"><span>${t("locked_offline")}</span><button class="btn small" data-page="premium">${t("refresh_license")}</button></div>`;
  return "";
}

/** "Update available" / "Updating…" banner, shown on every page. */
function updateBanner() {
  const u = S.status.update;
  if (!u) return "";
  if (S.updating || u.requested || u.phase) return `<div class="banner info">⏳ ${t("update_installing")}</div>`;
  if (!u.available) return "";
  const notes = getLanguage() === "bn" && u.available.notes_bn ? u.available.notes_bn : u.available.notes;
  return `<div class="banner ok row between"><span><strong>🎉 ${t("update_available", { v: esc(u.available.version) })}</strong>${
    notes ? `<br><small>${t("whats_new")}: ${esc(notes)}</small>` : ""
  }</span><button class="btn small primary" data-update-now>${t("update_now")}</button></div>`;
}

/** Starts an in-app update; the app reconnects by itself afterwards. */
export async function startUpdate() {
  try {
    await api("POST", "/api/update/install");
    S.updating = true;
    S.updatingFrom = S.status.version;
    S.updateStarted = Date.now();
    render();
  } catch (e) {
    toast(errMsg(e), true);
  }
}

/** After an update: say so when the new version answers, or if it failed. */
function followUpdate() {
  if (!S.updating || !S.status) return;
  const u = S.status.update || {};
  if (S.status.version !== S.updatingFrom) {
    S.updating = false;
    toast(t("update_done", { v: S.status.version }));
  } else if (!u.requested && !u.phase && /failed/.test(u.status || "")) {
    S.updating = false;
    toast(t("update_failed"), true);
  } else if (Date.now() - S.updateStarted > 10 * 60_000) {
    S.updating = false; // give up waiting quietly; the banner returns if still available
  }
}

function renderNav() {
  $("#tagline").textContent = t("tagline");
  $("#nav").innerHTML = PAGES.map(
    ([id, icon, key]) => `<li><button data-page="${id}" ${S.page === id ? 'aria-current="page"' : ""}><span aria-hidden="true">${icon}</span>${t(key)}</button></li>`,
  ).join("");
  $$("#nav button").forEach((b) =>
    b.addEventListener("click", () => {
      S.page = b.dataset.page;
      render();
      $("#main").focus();
    }),
  );
}

function render() {
  renderNav();
  const main = $("#main");
  if (S.offline || !S.status) {
    // While updating, the service restarts for a few seconds: that's expected.
    main.innerHTML = S.updating
      ? `<div class="banner info">⏳ ${t("update_installing")}</div>`
      : `<div class="banner danger">${t("agent_offline")}</div>`;
    return;
  }
  PAGES.find((p) => p[0] === S.page)[3](main);
  // Prepend the license banner (not on the premium page itself).
  if (S.page !== "premium") main.insertAdjacentHTML("afterbegin", licenseBanner());
  main.insertAdjacentHTML("afterbegin", updateBanner());
  $("[data-update-now]", main)?.addEventListener("click", startUpdate);
  $$("#main [data-page]").forEach((b) =>
    b.addEventListener("click", () => {
      S.page = b.dataset.page;
      render();
    }),
  );
  updateCountdowns();
  if (!cfg().settings.onboarded && !S.onboarding) startOnboarding();
}
S.render = render;

// ---------- onboarding ----------

const ob = { step: 0, language: "en", goal: "", presets: new Set(["adult", "gambling", "shorts"]), level: "mindful" };

function startOnboarding() {
  S.onboarding = true;
  ob.language = cfg().settings.language || "en";
  showStep();
}

function showStep() {
  const steps = `<div class="steps">${[0, 1, 2].map((i) => `<span class="${i <= ob.step ? "on" : ""}"></span>`).join("")}</div>`;
  let body = "";
  if (ob.step === 0) {
    body = `<h2>${t("ob_welcome")}</h2>
      <label>${t("language")}<select id="ob-lang">${languageOptions(ob.language)}</select></label>
      <label>${t("ob_goal_q")}<input type="text" id="ob-goal" maxlength="140" value="${esc(ob.goal)}" placeholder="${esc(t("goal_placeholder"))}"></label>`;
  } else if (ob.step === 1) {
    body = `<h2>${t("ob_presets_q")}</h2>
      <p class="hint">${t("ob_presets_hint")}</p>
      <div class="preset-grid">${S.status.presets.map((p) => `<button class="preset" data-p="${esc(p.id)}" aria-pressed="${ob.presets.has(p.id)}"><span class="icon">${p.icon}</span><strong>${esc(presetName(p))}</strong></button>`).join("")}</div>`;
  } else {
    body = `<h2>${t("ob_protection_q")}</h2>
      <div class="level-cards">${["off", "mindful", "committed"].map((lv) => `<button class="preset" data-lv="${lv}" aria-pressed="${ob.level === lv}"><strong>${t(`level_${lv}`)}${lv === "mindful" ? ` <span class="badge ok">${t("recommended")}</span>` : ""}</strong><small>${t(`level_${lv}_desc`, { hours: num(24), days: num(7) })}</small></button>`).join("")}</div>
      <p class="hint">${t("practice_mode")}</p>`;
  }
  const back = ob.step > 0 ? `<button class="btn" id="ob-back">${t("ob_back")}</button>` : "";
  const next = `<button class="btn primary" id="ob-next">${ob.step === 2 ? t("ob_finish") : t("ob_next")}</button>`;
  openModal(`${steps}${body}<div class="actions">${back}${next}</div>`, (m) => {
    $("#ob-lang", m)?.addEventListener("change", (e) => {
      ob.language = e.target.value;
      ob.goal = $("#ob-goal", m).value;
      import("/i18n.js").then((i) => { i.setLanguage(ob.language); render(); showStep(); });
    });
    $$("[data-p]", m).forEach((b) => b.addEventListener("click", () => {
      ob.presets.has(b.dataset.p) ? ob.presets.delete(b.dataset.p) : ob.presets.add(b.dataset.p);
      b.setAttribute("aria-pressed", ob.presets.has(b.dataset.p));
    }));
    $$("[data-lv]", m).forEach((b) => b.addEventListener("click", () => {
      ob.level = b.dataset.lv;
      $$("[data-lv]", m).forEach((x) => x.setAttribute("aria-pressed", x === b));
    }));
    $("#ob-back", m)?.addEventListener("click", () => { ob.step--; showStep(); });
    $("#ob-next", m).addEventListener("click", async () => {
      if (ob.step === 0) ob.goal = $("#ob-goal", m).value.trim();
      if (ob.step < 2) { ob.step++; return showStep(); }
      await finishOnboarding();
    });
  });
}

async function finishOnboarding() {
  try {
    // Language first, so ready-made lists get names in the chosen language.
    const c1 = clone(cfg());
    c1.settings.language = ob.language;
    c1.settings.goal = ob.goal;
    // Safe defaults that close the two easiest ways around blocking without
    // getting in anyone's way: browsers that can't be filtered at all (Tor
    // ignores the hosts file) are closed while blocking, and search engines
    // stay in SafeSearch (otherwise image search shows what the lists block).
    c1.settings.block_unsupported_browsers = true;
    c1.settings.force_safe_search = true;
    // The extension installs itself where the browser allows it (Chrome,
    // Brave, Vivaldi), and private windows - where extensions often don't run -
    // are turned off. Neither gets in the way of normal browsing.
    c1.settings.managed_extension = true;
    c1.settings.block_private_windows = true;
    await api("PUT", "/api/config", c1);
    for (const id of ob.presets) await addPreset(id);
    await refresh();
    const c2 = clone(cfg());
    c2.settings.onboarded = true;
    c2.protection.level = ob.level;
    await api("PUT", "/api/config", c2);
    await refresh();
    S.onboarding = false;
    closeModal();
    render();
    toast(t("ob_done"));
  } catch (e) {
    toast(errMsg(e), true);
  }
}

// ---------- polling ----------

function userIsEditing() {
  const a = document.activeElement;
  return modalOpen() || (a && ["INPUT", "TEXTAREA", "SELECT"].includes(a.tagName));
}

/** The window keeps running the page it loaded; after the agent updated
 * itself (automatically or from another window) load the new version's page,
 * staying on the same tab. Waits while the user is typing in a form. */
function reloadIfAgentUpdated() {
  if (!S.status || S.onboarding) return false;
  S.loadedVersion ??= S.status.version;
  if (S.status.version === S.loadedVersion || userIsEditing()) return false;
  try {
    sessionStorage.setItem("cg-updated-to", S.status.version);
    sessionStorage.setItem("cg-page", S.page);
  } catch {}
  location.reload();
  return true;
}

async function poll() {
  await refresh();
  if (reloadIfAgentUpdated()) return;
  followUpdate();
  if (S.updating) return render();
  // Only the home page refreshes itself; forms elsewhere must not be wiped while typing.
  if ((S.page === "home" || S.offline) && !userIsEditing()) render();
}

await refresh();
S.loadedVersion = S.status?.version;
try {
  const updated = sessionStorage.getItem("cg-updated-to");
  const page = sessionStorage.getItem("cg-page");
  sessionStorage.removeItem("cg-updated-to");
  sessionStorage.removeItem("cg-page");
  if (page) S.page = page;
  if (updated) setTimeout(() => toast(t("update_done", { v: updated })), 300);
} catch {}
render();
setInterval(poll, 4000);
setInterval(updateCountdowns, 1000);
