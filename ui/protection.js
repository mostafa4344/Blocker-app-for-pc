// Protection (uninstall/settings) and general settings.
import { S, $, $$, t, num, esc, api, act, cfg, clone, fmtTime, openModal, challengeModal, saveConfig, toast, errMsg } from "/core.js";

const LEVELS = ["off", "mindful", "committed", "partner"];
const FILTERS = ["youtube_shorts", "youtube_recommendations", "youtube_comments", "facebook_feed", "facebook_reels", "instagram_reels", "x_feed", "linkedin_feed"];

function levelDesc(level, p) {
  return t(`level_${level}_desc`, { hours: num(p.cooldown_hours), days: num(p.partner_timeout_days) });
}

/** Stricter (or equal) settings apply directly; anything looser becomes a request. */
async function applyProtection(next) {
  const res = await saveConfig((c) => (c.protection = next));
  if (res === true) return;
  if (res?.code === "protection_requires_request") {
    await act(() => api("POST", "/api/requests", { kind: { type: "change_protection", new: next } }), t("protection_lower_note"));
  }
}

function requestRow(r) {
  const kind = t(`kind_${r.kind.type}`);
  const status = t(`st_${r.status}`);
  const actions = [];
  if (r.status === "needs_challenge") actions.push(`<button class="btn small primary" data-challenge="${esc(r.id)}">${t("type_challenge")}</button>`);
  if (r.status === "approved" && r.kind.type !== "uninstall") actions.push(`<button class="btn small primary" data-apply="${esc(r.id)}">${t("apply")}</button>`);
  if (["needs_challenge", "pending", "approved"].includes(r.status)) actions.push(`<button class="btn small" data-cancel="${esc(r.id)}">${t("cancel")}</button>`);
  const info = [];
  if (r.status === "pending" && r.needs_partner) info.push(t("waiting_partner"));
  if (r.status === "pending" && r.available_at) info.push(`${t("available_at")}: ${fmtTime(r.available_at)}`);
  if (r.status === "approved" && r.kind.type === "uninstall") info.push(t("uninstall_ready"));
  const badge = r.status === "approved" ? "ok" : r.status === "denied" ? "danger" : "";
  return `<div class="item"><div><div class="row"><strong>${kind}</strong><span class="badge ${badge}">${status}</span></div>
    <div class="meta">${esc(info.join(" · "))}${r.reason ? ` — “${esc(r.reason)}”` : ""}</div></div><div class="row">${actions.join("")}</div></div>`;
}

export function renderProtection(main) {
  const p = cfg().protection;
  const reqs = S.status.requests.slice().reverse();
  main.innerHTML = `
    <div class="page-head"><div><h1>${t("protection_title")}</h1><p class="muted">${t("protection_intro")}</p></div></div>
    <div class="stack">
      <div class="level-cards">${LEVELS.map((lv) => `
        <div class="level ${p.level === lv ? "current" : ""}">
          <h3>${t(`level_${lv}`)}</h3><p class="muted">${levelDesc(lv, p)}</p>
          ${p.level === lv ? `<span class="badge ok">${t("current")}</span>` : `<button class="btn small" data-level="${lv}">${t("choose")}</button>`}
        </div>`).join("")}</div>

      <section class="card stack">
        <div class="grid two">
          <label>${t("cooldown_hours")}<select id="p-cool">${[24, 48, 72].map((h) => `<option value="${h}" ${p.cooldown_hours === h ? "selected" : ""}>${num(h)}</option>`).join("")}</select></label>
          <label>${t("partner_timeout")}<input type="number" id="p-timeout" min="1" max="30" value="${p.partner_timeout_days}"></label>
          <label>${t("partner_name")}<input type="text" id="p-name" value="${esc(p.partner?.name ?? "")}"></label>
          <label>${t("partner_email")}<input type="email" id="p-email" value="${esc(p.partner?.email ?? "")}"></label>
        </div>
        <label class="check"><input type="checkbox" id="p-tamper" ${p.tamper_alerts ? "checked" : ""}>${t("tamper_alerts")}</label>
        <p class="hint">${t("protection_lower_note")}</p>
        <div><button class="btn primary" id="p-save">${t("save")}</button></div>
      </section>

      <section class="card stack">
        <div class="row between"><h2>${t("requests")}</h2></div>
        <div class="row"><input type="text" id="u-reason" placeholder="${esc(t("reason"))}"><button class="btn danger" id="u-req">${t("request_uninstall")}</button></div>
        <div class="list">${reqs.length === 0 ? `<p class="muted">${t("no_data")}</p>` : reqs.map(requestRow).join("")}</div>
      </section>
    </div>`;

  const formProtection = (level) => {
    const name = $("#p-name", main).value.trim();
    const email = $("#p-email", main).value.trim();
    return {
      ...clone(p),
      level,
      cooldown_hours: Number($("#p-cool", main).value),
      partner_timeout_days: Math.min(30, Math.max(1, Number($("#p-timeout", main).value) || 7)),
      partner: email ? { name: name || email, email } : null,
      tamper_alerts: $("#p-tamper", main).checked,
    };
  };
  $$("[data-level]", main).forEach((b) => b.addEventListener("click", () => {
    const next = formProtection(b.dataset.level);
    if (next.level === "partner" && !next.partner) return toast(t("partner_email"), true);
    applyProtection(next);
  }));
  $("#p-save", main).addEventListener("click", () => applyProtection(formProtection(p.level)));
  $("#u-req", main).addEventListener("click", () => act(() => api("POST", "/api/requests", { kind: { type: "uninstall" }, reason: $("#u-reason", main).value }), t("saved")));
  $$("[data-cancel]", main).forEach((b) => b.addEventListener("click", () => act(() => api("POST", `/api/requests/${b.dataset.cancel}/cancel`))));
  $$("[data-apply]", main).forEach((b) => b.addEventListener("click", () => act(() => api("POST", `/api/requests/${b.dataset.apply}/apply`), t("saved"))));
  $$("[data-challenge]", main).forEach((b) => b.addEventListener("click", () => {
    const id = b.dataset.challenge;
    challengeModal(`request:${id}`, async (answer) => {
      try {
        await api("POST", `/api/requests/${id}/challenge`, { answer });
        await act(async () => {});
        return true;
      } catch (e) {
        toast(errMsg(e), true);
        return false;
      }
    });
  }));
}

export function renderSettings(main) {
  const st = cfg().settings;
  const dev = S.status.device;
  main.innerHTML = `
    <div class="page-head"><h1>${t("settings_title")}</h1></div>
    <div class="stack">
      
      <section class="card stack">
        <label>${t("language")}<select id="s-lang"><option value="bn" ${st.language === "bn" ? "selected" : ""}>বাংলা</option><option value="en" ${st.language === "en" ? "selected" : ""}>English</option></select></label>
        <label>${t("goal")}<input type="text" id="s-goal" maxlength="140" value="${esc(st.goal)}" placeholder="${esc(t("goal_placeholder"))}"></label>
        <label class="check"><input type="checkbox" id="s-ext" ${st.require_browser_extension ? "checked" : ""}>${t("require_ext")}</label>
      </section>
      <section class="card stack">
        <h2>${t("filters")}</h2>
        ${FILTERS.map((f) => `<label class="check"><input type="checkbox" data-filter="${f}" ${st.distraction_filters.includes(f) ? "checked" : ""}>${t(`f_${f}`)}</label>`).join("")}
      </section>
      <section class="card stack">
        <label>${t("backend_url")}<span class="hint">${t("backend_hint")}</span><input type="url" id="s-backend" value="${esc(st.backend_url)}" placeholder="https://"></label>
        ${dev.last_sync_ok ? `<span class="badge ok">${t("sync_ok")}: ${fmtTime(dev.last_sync_ok)}</span>` : ""}
        ${dev.last_error ? `<span class="badge danger">${t("sync_error")}: ${esc(dev.last_error)}</span>` : ""}
      </section>
      <div><button class="btn primary" id="s-save">${t("save")}</button></div>
      <section class="card stack"><h2>${t("extension_title")}</h2><p class="muted">${t("extension_help")}</p></section>
      <p class="muted">ContentGuard v${esc(S.status.version)}</p>
    </div>`;
  $("#s-save", main).addEventListener("click", () => saveConfig((c) => {
    c.settings.language = $("#s-lang", main).value;
    c.settings.goal = $("#s-goal", main).value.trim();
    c.settings.require_browser_extension = $("#s-ext", main).checked;
    c.settings.distraction_filters = $$("[data-filter]", main).filter((x) => x.checked).map((x) => x.dataset.filter);
    c.settings.backend_url = $("#s-backend", main).value.trim();
  }));
}
