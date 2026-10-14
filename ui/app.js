// FocusFort UI entry point: navigation, rendering, onboarding and polling.
import { S, $, $$, t, num, esc, api, cfg, clone, refresh, openModal, closeModal, modalOpen, updateCountdowns, toast, errMsg } from "/core.js";
import { renderHome } from "/home.js";
import { renderLists, renderSchedules, renderLimits } from "/editors.js";
import { renderProtection, renderSettings } from "/protection.js";

const PAGES = [
  ["home", "🏠", "nav_home", renderHome],
  ["lists", "🚫", "nav_lists", renderLists],
  ["schedules", "🗓️", "nav_schedules", renderSchedules],
  ["limits", "⏳", "nav_limits", renderLimits],
  ["protection", "🛡️", "nav_protection", renderProtection],
  ["settings", "⚙️", "nav_settings", renderSettings],
];

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
    main.innerHTML = `<div class="banner danger">${t("agent_offline")}</div>`;
    return;
  }
  PAGES.find((p) => p[0] === S.page)[3](main);
  updateCountdowns();
  if (!cfg().settings.onboarded && !S.onboarding) startOnboarding();
}
S.render = render;

// ---------- onboarding ----------

const ob = { step: 0, language: "bn", goal: "", presets: new Set(["social", "shorts"]), level: "off" };

function startOnboarding() {
  S.onboarding = true;
  ob.language = cfg().settings.language || "bn";
  showStep();
}

function showStep() {
  const bn = ob.language === "bn";
  const steps = `<div class="steps">${[0, 1, 2].map((i) => `<span class="${i <= ob.step ? "on" : ""}"></span>`).join("")}</div>`;
  let body = "";
  if (ob.step === 0) {
    body = `<h2>${t("ob_welcome")}</h2>
      <label>${t("language")}<select id="ob-lang"><option value="bn" ${bn ? "selected" : ""}>বাংলা</option><option value="en" ${bn ? "" : "selected"}>English</option></select></label>
      <label>${t("ob_goal_q")}<input type="text" id="ob-goal" maxlength="140" value="${esc(ob.goal)}" placeholder="${esc(t("goal_placeholder"))}"></label>`;
  } else if (ob.step === 1) {
    body = `<h2>${t("ob_presets_q")}</h2>
      <div class="preset-grid">${S.status.presets.map((p) => `<button class="preset" data-p="${esc(p.id)}" aria-pressed="${ob.presets.has(p.id)}"><span class="icon">${p.icon}</span><strong>${esc(bn ? p.name_bn : p.name_en)}</strong></button>`).join("")}</div>`;
  } else {
    body = `<h2>${t("ob_protection_q")}</h2>
      <div class="level-cards">${["off", "mindful", "committed"].map((lv) => `<button class="preset" data-lv="${lv}" aria-pressed="${ob.level === lv}"><strong>${t(`level_${lv}`)}</strong><small>${t(`level_${lv}_desc`, { hours: num(24), days: num(7) })}</small></button>`).join("")}</div>
      <p class="hint">${t("practice_mode")}</p>`;
  }
  const back = ob.step > 0 ? `<button class="btn" id="ob-back">${t("ob_back")}</button>` : "";
  const next = `<button class="btn primary" id="ob-next">${ob.step === 2 ? t("ob_finish") : t("ob_next")}</button>`;
  openModal(`${steps}${body}<div class="actions">${back}${next}</div>`, (m) => {
    $("#ob-lang", m)?.addEventListener("change", (e) => {
      ob.language = e.target.value;
      ob.goal = $("#ob-goal", m).value;
      import("/i18n.js").then((i) => { i.setLanguage(ob.language); showStep(); renderNav(); });
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
    await api("PUT", "/api/config", c1);
    for (const id of ob.presets) await api("POST", `/api/presets/${id}`);
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

async function poll() {
  await refresh();
  // Only the home page refreshes itself; forms elsewhere must not be wiped while typing.
  if ((S.page === "home" || S.offline) && !userIsEditing()) render();
}

await refresh();
render();
setInterval(poll, 4000);
setInterval(updateCountdowns, 1000);
