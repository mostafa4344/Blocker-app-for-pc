// Protection (uninstall/settings), general settings, and the Premium/account page.
import { S, $, $$, t, num, esc, api, act, cfg, clone, refresh, fmtTime, openModal, closeModal, challengeModal, saveConfig, toast, errMsg } from "/core.js";

const LEVELS = ["off", "mindful", "committed"];
const FILTERS = ["youtube_shorts", "youtube_recommendations", "youtube_comments", "facebook_feed", "facebook_reels", "instagram_reels", "x_feed", "linkedin_feed"];

// Time-delay options, in hours. The last one (~100 years) is shown as "unlimited".
const UNLIMITED_HOURS = 876000;
const DELAY_OPTIONS = [24, 72, 168, 336, 720, 2160, 8760, UNLIMITED_HOURS];

function delayLabel(hours) {
  if (hours >= UNLIMITED_HOURS) return t("delay_unlimited");
  if (hours % 24 === 0) return t("delay_days", { n: num(hours / 24) });
  return t("delay_hours", { n: num(hours) });
}

function levelDesc(level, p) {
  if (level === "committed") return t("level_committed_desc", { hours: delayLabel(p.cooldown_hours) });
  return t(`level_${level}_desc`, { hours: delayLabel(p.cooldown_hours), days: num(p.partner_timeout_days) });
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
  if (r.status === "pending" && r.available_at) info.push(`${t("available_at")}: ${fmtTime(r.available_at)}`);
  if (r.status === "approved" && r.kind.type === "uninstall") info.push(t("uninstall_ready"));
  const badge = r.status === "approved" ? "ok" : r.status === "denied" ? "danger" : "";
  return `<div class="item"><div><div class="row"><strong>${kind}</strong><span class="badge ${badge}">${status}</span></div>
    <div class="meta">${esc(info.join(" · "))}${r.reason ? ` — “${esc(r.reason)}”` : ""}</div></div><div class="row">${actions.join("")}</div></div>`;
}

export function renderProtection(main) {
  const p = cfg().protection;
  const reqs = S.status.requests.slice().reverse();
  // Make sure the current cooldown is selectable even if it is a custom value.
  const opts = DELAY_OPTIONS.includes(p.cooldown_hours) ? DELAY_OPTIONS : [p.cooldown_hours, ...DELAY_OPTIONS];
  main.innerHTML = `
    <div class="page-head"><div><h1>${t("protection_title")}</h1><p class="muted">${t("protection_intro")}</p></div></div>
    <div class="stack">
      <div class="level-cards">${LEVELS.map((lv) => `
        <div class="level ${p.level === lv ? "current" : ""}">
          <h3>${t(`level_${lv}`)}</h3><p class="muted">${levelDesc(lv, p)}</p>
          ${p.level === lv ? `<span class="badge ok">${t("current")}</span>` : `<button class="btn small" data-level="${lv}">${t("choose")}</button>`}
        </div>`).join("")}</div>

      <section class="card stack">
        <label>${t("delay_label")}
          <select id="p-cool">${opts.map((h) => `<option value="${h}" ${p.cooldown_hours === h ? "selected" : ""}>${delayLabel(h)}</option>`).join("")}</select>
        </label>
        <p class="hint">${t("protection_lower_note")}</p>
        <div><button class="btn primary" id="p-save">${t("save")}</button></div>
      </section>

      <section class="card stack">
        <div class="row between"><h2>${t("requests")}</h2></div>
        <div class="row"><input type="text" id="u-reason" placeholder="${esc(t("reason"))}"><button class="btn danger" id="u-req">${t("request_uninstall")}</button></div>
        <div class="list">${reqs.length === 0 ? `<p class="muted">${t("no_data")}</p>` : reqs.map(requestRow).join("")}</div>
      </section>
    </div>`;

  const formProtection = (level) => ({
    ...clone(p),
    level,
    cooldown_hours: Number($("#p-cool", main).value),
    partner: null,
  });
  $$("[data-level]", main).forEach((b) => b.addEventListener("click", () => applyProtection(formProtection(b.dataset.level))));
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

// ---------- Premium / account ----------

function statusBadge(st) {
  const cls = st === "premium" ? "ok" : st === "trial" ? "" : "danger";
  return `<span class="badge ${cls}">${t(`st_${st}`)}</span>`;
}

export function activateModal() {
  openModal(
    `<h2>${t("activate_title")}</h2><p class="muted">${t("activate_help")}</p>
     <label>${t("activate_code")}<input type="text" id="code" autocomplete="off" placeholder="PREMIUM-..."></label>
     <p class="hint">${t("buy_help")}</p>
     <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="go">${t("activate_btn")}</button></div>`,
    (m) => $("#go", m).addEventListener("click", async () => {
      const code = $("#code", m).value.trim();
      if (!code) return;
      if (await act(() => api("POST", "/api/license/activate", { code }), t("activated"))) closeModal();
    }),
  );
}

export function renderLicense(main) {
  const lic = S.status.license;
  const st = lic.status.state;
  const until = lic.premium_until ? fmtTime(lic.premium_until) : t("premium_lifetime");
  main.innerHTML = `
    <div class="page-head"><h1>${t("license_title")}</h1></div>
    <div class="stack">
      <section class="card stack">
        <div class="row between"><h2>${t("license_status_label")}</h2>${statusBadge(st)}</div>
        ${st === "trial" ? `<p>${t("trial_days_left", { days: num(lic.status.days_left) })}</p>` : ""}
        ${st === "premium" ? `<p>${t("premium_active")} · ${t("premium_until_label")}: ${until}</p>` : ""}
        ${st === "expired" ? `<p class="muted">${t("trial_expired")}</p>` : ""}
        ${st === "locked_offline" ? `<p class="muted">${t("locked_offline")}</p>` : ""}
        ${lic.last_checked ? `<small>${t("last_checked_label")}: ${fmtTime(lic.last_checked)}</small>` : ""}
        ${lic.last_error ? `<small class="badge danger">${esc(lic.last_error)}</small>` : ""}
        <div class="row">
          <button class="btn primary" id="activate">${t("get_premium")}</button>
          <button class="btn" id="refresh">${t("refresh_license")}</button>
        </div>
      </section>
      <section class="card stack"><h2>${t("buy_title")}</h2><p class="muted">${t("buy_help")}</p></section>
    </div>`;
  $("#activate", main).addEventListener("click", activateModal);
  $("#refresh", main).addEventListener("click", () => act(() => api("POST", "/api/license/refresh"), t("saved")));
}

// ---------- Settings ----------

export function renderSettings(main) {
  const st = cfg().settings;
  const lic = S.status.license;
  const browsers = S.status.browsers;
  const nice = (name) => name.replace(/\b\w/g, (c) => c.toUpperCase());
  main.innerHTML = `
    <div class="page-head"><h1>${t("settings_title")}</h1></div>
    <div class="stack">
      <section class="card stack">
        <label>${t("language")}<select id="s-lang"><option value="bn" ${st.language === "bn" ? "selected" : ""}>বাংলা</option><option value="en" ${st.language === "en" ? "selected" : ""}>English</option></select></label>
        <label>${t("goal")}<input type="text" id="s-goal" maxlength="140" value="${esc(st.goal)}" placeholder="${esc(t("goal_placeholder"))}"></label>
      </section>

      <section class="card stack">
        <h2>${t("browsers_title")}</h2>
        <label class="check"><input type="checkbox" id="s-ext" ${st.require_browser_extension ? "checked" : ""}>${t("require_ext")}</label>
        <label class="check"><input type="checkbox" id="s-unsup" ${st.block_unsupported_browsers ? "checked" : ""}><span>${t("block_unsupported")}<br><span class="hint">${t("block_unsupported_help")}</span></span></label>
        <label class="check"><input type="checkbox" id="s-safe" ${st.force_safe_search ? "checked" : ""}><span>${t("safe_search")}<br><span class="hint">${t("safe_search_help")}</span></span></label>
        <div><strong>${t("supported_browsers")}</strong><div class="chips">${browsers.supported.map((b) => `<span class="badge ok">${esc(nice(b))}</span>`).join("")}</div></div>
        <div><strong>${t("unsupported_browsers")}</strong><div class="chips">${browsers.unsupported.map((b) => `<span class="badge">${esc(nice(b))}</span>`).join("")}</div></div>
      </section>

      <section class="card stack">
        <h2>${t("filters")}</h2>
        ${FILTERS.map((f) => `<label class="check"><input type="checkbox" data-filter="${f}" ${st.distraction_filters.includes(f) ? "checked" : ""}>${t(`f_${f}`)}</label>`).join("")}
      </section>

      <div><button class="btn primary" id="s-save">${t("save")}</button></div>

      <details class="card stack"><summary>${t("backend_url")}</summary>
        <label>${t("backend_url")}<span class="hint">${t("backend_hint")}</span><input type="url" id="s-backend" value="${esc(st.backend_url)}" placeholder="https://"></label>
        <label>${t("backend_key_label")}<input type="text" id="s-backend-key" value="${esc(st.backend_key)}"></label>
        <label>${t("install_id_label")}<input type="text" value="${esc(lic.install_id || "")}" readonly onclick="this.select()"></label>
        ${lic.last_checked ? `<span class="badge ok">${t("last_checked_label")}: ${fmtTime(lic.last_checked)}</span>` : ""}
        ${lic.last_error ? `<span class="badge danger">${esc(lic.last_error)}</span>` : ""}
      </details>

      <details class="card stack"><summary>${t("backup_title")}</summary>
        <p class="hint">${t("backup_help")}</p>
        <textarea id="s-backup" spellcheck="false" readonly>${esc(JSON.stringify(cfg(), null, 2))}</textarea>
        <div class="row"><button class="btn" id="s-copy">${t("backup_copy")}</button><button class="btn" id="s-import">${t("backup_import")}</button></div>
      </details>

      <section class="card stack"><h2>${t("extension_title")}</h2><p class="muted">${t("extension_help")}</p></section>
      <p class="muted">Content Guard v${esc(S.status.version)}</p>
    </div>`;
  $("#s-save", main).addEventListener("click", () => saveConfig((c) => {
    c.settings.language = $("#s-lang", main).value;
    c.settings.goal = $("#s-goal", main).value.trim();
    c.settings.require_browser_extension = $("#s-ext", main).checked;
    c.settings.block_unsupported_browsers = $("#s-unsup", main).checked;
    c.settings.force_safe_search = $("#s-safe", main).checked;
    c.settings.distraction_filters = $$("[data-filter]", main).filter((x) => x.checked).map((x) => x.dataset.filter);
    c.settings.backend_url = $("#s-backend", main).value.trim();
    c.settings.backend_key = $("#s-backend-key", main).value.trim();
  }));
  $("#s-copy", main)?.addEventListener("click", () => {
    const ta = $("#s-backup", main);
    ta.select();
    navigator.clipboard?.writeText(ta.value).then(() => toast(t("saved")), () => {});
  });
  $("#s-import", main)?.addEventListener("click", () => {
    openModal(
      `<h2>${t("backup_import")}</h2><p class="muted">${t("backup_import_help")}</p>
       <textarea id="imp" spellcheck="false" placeholder='{ "block_lists": [...] }'></textarea>
       <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="imp-go">${t("backup_import")}</button></div>`,
      (m) => $("#imp-go", m).addEventListener("click", async () => {
        let parsed;
        try {
          parsed = JSON.parse($("#imp", m).value);
        } catch {
          return toast(t("backup_bad_json"), true);
        }
        try {
          await api("PUT", "/api/config", parsed);
          await refresh();
          S.render();
          closeModal();
          toast(t("saved"));
        } catch (e) {
          toast(errMsg(e), true);
        }
      }),
    );
  });
}
