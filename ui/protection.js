// Protection (uninstall/settings), general settings, and the Premium/account page.
import { S, $, $$, t, num, esc, languageOptions, api, act, cfg, clone, refresh, fmtTime, fmtDate, listName, openModal, closeModal, challengeModal, saveConfig, toast, errMsg } from "/core.js";
import { tip } from "/help.js";

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
  if (r.kind.type === "add_exceptions") info.unshift(`${listName(r.kind.list_id)}: ${[...r.kind.sites, ...r.kind.apps].join(", ")}`);
  const badge = r.status === "approved" ? "ok" : r.status === "denied" ? "danger" : "";
  return `<div class="item"><div><div class="row"><strong>${kind}</strong><span class="badge ${badge}">${status}</span></div>
    <div class="meta">${esc(info.join(" · "))}${r.reason ? ` — “${esc(r.reason)}”` : ""}</div></div><div class="row">${actions.join("")}</div></div>`;
}

export function renderProtection(main) {
  const p = cfg().protection;
  // Only requests still in progress; cancelled, expired, denied and used ones are history.
  const ACTIVE = ["needs_challenge", "pending", "approved"];
  const reqs = S.status.requests.filter((r) => ACTIVE.includes(r.status)).reverse();
  // Make sure the current cooldown is selectable even if it is a custom value.
  const opts = DELAY_OPTIONS.includes(p.cooldown_hours) ? DELAY_OPTIONS : [p.cooldown_hours, ...DELAY_OPTIONS];
  main.innerHTML = `
    <div class="page-head"><div><h1>${t("protection_title")} ${tip("protection_levels")}</h1><p class="muted">${t("protection_intro")}</p></div></div>
    <div class="stack">
      <div class="level-cards">${LEVELS.map((lv) => `
        <div class="level ${p.level === lv ? "current" : ""}">
          <h3>${t(`level_${lv}`)}</h3><p class="muted">${levelDesc(lv, p)}</p>
          ${p.level === lv ? `<span class="badge ok">${t("current")}</span>` : `<button class="btn small" data-level="${lv}">${t("choose")}</button>`}
        </div>`).join("")}</div>

      <section class="card stack">
        <label>${t("delay_label")} ${tip("protection_delay")}<span class="hint">${t("delay_hint")}</span>
          <select id="p-cool">${opts.map((h) => `<option value="${h}" ${p.cooldown_hours === h ? "selected" : ""}>${delayLabel(h)}</option>`).join("")}</select>
        </label>
        <div><button class="btn primary" id="p-save">${t("save")}</button></div>
      </section>

      <section class="card stack">
        <h2>${t("uninstall_title")} ${tip("uninstall")}</h2>
        <p class="muted">${t(p.level === "off" ? "uninstall_help_off" : "uninstall_help")}</p>
        <div class="row"><input type="text" id="u-reason" placeholder="${esc(t("reason"))}"><button class="btn danger" id="u-req">${t("request_uninstall")}</button></div>
      </section>

      ${reqs.length === 0 ? "" : `<section class="card stack">
        <h2>${t("requests")} ${tip("requests")}</h2>
        <div class="list">${reqs.map(requestRow).join("")}</div>
      </section>`}
    </div>`;

  const formProtection = (level) => ({
    ...clone(p),
    level,
    cooldown_hours: Number($("#p-cool", main).value),
    partner: null,
  });
  $$("[data-level]", main).forEach((b) => b.addEventListener("click", () => applyProtection(formProtection(b.dataset.level))));
  $("#p-save", main).addEventListener("click", () => applyProtection(formProtection(p.level)));
  $("#u-req", main).addEventListener("click", () => act(() => api("POST", "/api/requests", { kind: { type: "uninstall" }, reason: $("#u-reason", main).value }), t("request_created")));
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

const METHODS = [["bkash", "bKash"], ["nagad", "Nagad"], ["rocket", "Rocket"]];
const buy = { plan: "", method: "" };

/** Plans from the server's "plans" table; older servers only send fixed keys. */
function planList(pay) {
  if (Array.isArray(pay?.plans)) return pay.plans.filter((p) => p && p.code && p.price);
  return [["monthly", 1], ["yearly", 12], ["lifetime", null]]
    .filter(([k]) => pay?.[k])
    .map(([code, months]) => ({ code, months, price: pay[code], name: "", popular: code === "yearly" }));
}

/** A plan's label: its own name, else Monthly / Yearly / Lifetime / "N months". */
function planLabel(plan) {
  if (plan.name) return plan.name;
  if (plan.months == null) return t("plan_lifetime");
  if (plan.months === 1) return t("plan_monthly");
  if (plan.months === 12) return t("plan_yearly");
  return t("plan_n_months", { n: num(plan.months) });
}

/** Label for a plan code (e.g. of a pending request), even if it is no longer offered. */
function codeLabel(code, plans) {
  const p = plans.find((x) => x.code === code);
  if (p) return planLabel(p);
  const m = /^(\d+)m$/.exec(code || "");
  return m ? t("plan_n_months", { n: num(Number(m[1])) }) : t(`plan_${code}`);
}

export function renderLicense(main) {
  // Check with the server when the page opens (e.g. right after buying), at
  // most once a minute; the background check only runs every few hours.
  if (Date.now() - (S.licenseCheckedAt || 0) > 60_000) {
    S.licenseCheckedAt = Date.now();
    api("POST", "/api/license/refresh")
      .then(refresh)
      .then(() => S.page === "premium" && S.render())
      .catch(() => {});
  }
  const lic = S.status.license;
  const st = lic.status.state;
  const until = lic.premium_until ? fmtDate(lic.premium_until) : t("premium_lifetime");
  const pay = lic.pay || null;
  const methods = pay ? METHODS.filter(([k]) => pay[k]) : [];
  if (!methods.some(([k]) => k === buy.method)) buy.method = methods[0]?.[0] || "";
  const pending = st !== "premium" ? lic.pending_purchase : null;
  const plans = planList(pay);
  if (!plans.some((p) => p.code === buy.plan)) buy.plan = (plans.find((p) => p.popular) || plans[0])?.code || "";
  const price = plans.find((p) => p.code === buy.plan)?.price || "";
  const number = pay && buy.method ? pay[buy.method] : "";
  const methodName = (METHODS.find(([k]) => k === buy.method) || [, ""])[1];

  const buyCard = !pay || plans.length === 0
    ? `<section class="card stack"><h2>${t("buy_title")}</h2><p class="muted">${t("buy_help_contact")}</p><div class="row"><button class="btn" id="pay-retry">${t("refresh_license")}</button></div></section>`
    : `<section class="card stack">
        <h2>${t(st === "premium" ? "renew_title" : "buy_title")} ${tip("premium_buy")}</h2>
        <div class="stack"><strong>${t("buy_step1")}</strong>
          <div class="plan-grid">${plans.map((p) => `
            <button class="plan" data-plan="${esc(p.code)}" aria-pressed="${buy.plan === p.code}">
              <span>${esc(planLabel(p))}</span><strong>${esc(p.price)}</strong>
              ${p.popular ? `<small class="badge ok">${t("popular")}</small>` : ""}
            </button>`).join("")}</div></div>
        <div class="stack"><strong>${t("buy_step2")}</strong>
          <div class="chips">${methods.map(([k, n]) => `<button class="chip" data-method="${k}" aria-pressed="${buy.method === k}">${n}</button>`).join("")}</div>
          <div class="paybox">
            <p>${t("send_money_to", { method: methodName, price: esc(price) })}</p>
            <div class="row"><span class="paynum" id="pay-num">${esc(number)}</span><button class="btn small" id="copy-num">${t("copy_id")}</button></div>
            <p class="hint">${t("send_money_hint")}</p>
          </div></div>
        <div class="stack"><strong>${t("buy_step3")}</strong>
          <label>${t("trx_label")} ${tip("trx")}<input type="text" id="trx" maxlength="40" autocomplete="off" spellcheck="false" placeholder="${t("trx_placeholder")}"></label>
          <label>${t("contact_label")}<input type="tel" id="contact" maxlength="20" autocomplete="tel" placeholder="01XXXXXXXXX"></label>
          <div><button class="btn primary big" id="send">${t("submit_request")}</button></div>
        </div>
      </section>`;

  main.innerHTML = `
    <div class="page-head"><h1>${t("license_title")}</h1></div>
    <div class="stack">
      <section class="card stack">
        <div class="row between"><h2>${t("license_status_label")} ${tip("premium_status")}</h2>${statusBadge(st)}</div>
        ${st === "trial" ? `<p>${t("trial_days_left", { days: num(lic.status.days_left) })}</p>` : ""}
        ${st === "premium" ? `<p>✅ ${t("premium_active")} · ${t("premium_until_label")}: ${until}</p>` : ""}
        ${st === "expired" && !lic.device_mismatch ? `<p class="muted">${t("trial_expired")}</p>` : ""}
        ${st === "locked_offline" ? `<p class="muted">${t("locked_offline")}</p>` : ""}
        ${lic.device_mismatch ? `<p class="banner danger">${t("err_license_device")}</p>` : ""}
        ${lic.last_error && st !== "premium" && !lic.device_mismatch ? `<small class="muted">${t("err_license_offline")}</small>` : ""}
      </section>
      ${pending ? `<div class="banner info">⏳ ${t("request_pending", { plan: esc(codeLabel(pending.plan, plans)), trx: esc(pending.txn_id) })}</div>` : ""}
      ${buyCard}
      ${lic.purchase_url ? `<section class="card stack"><h2>${t("card_title")}</h2><p class="muted">${t("buy_help")}</p><div><button class="btn" id="buy">${t("get_premium")}</button></div></section>` : ""}
      <section class="card stack">
        <div class="row"><button class="btn" id="activate">${t("have_code")}</button>${tip("activation_code")}<button class="btn" id="refresh">${t("refresh_license")}</button></div>
        <label>${t("install_id_label")} ${tip("device_id")}<div class="row"><input type="text" id="dev-id" value="${esc(lic.install_id || "")}" readonly spellcheck="false"><button class="btn" id="copy-id">${t("copy_id")}</button></div></label>
      </section>
    </div>`;

  const copy = (text) => navigator.clipboard?.writeText(text).then(() => toast(t("copied")), () => {});
  $$("[data-plan]", main).forEach((x) => x.addEventListener("click", () => { buy.plan = x.dataset.plan; S.render(); }));
  $$("[data-method]", main).forEach((x) => x.addEventListener("click", () => { buy.method = x.dataset.method; S.render(); }));
  $("#copy-num", main)?.addEventListener("click", () => copy(number));
  $("#send", main)?.addEventListener("click", async () => {
    const txn_id = $("#trx", main).value.replace(/\s+/g, "").toUpperCase();
    if (!/^[A-Z0-9]{4,40}$/.test(txn_id)) return toast(t("err_invalid_txn"), true);
    const ok = await act(() => api("POST", "/api/purchase", { plan: buy.plan, method: buy.method, txn_id, contact: $("#contact", main).value }), t("request_sent"));
    if (ok) S.licenseCheckedAt = Date.now();
  });
  $("#buy", main)?.addEventListener("click", () => {
    const u = new URL(lic.purchase_url);
    u.searchParams.set("id", lic.install_id || "");
    window.open(u.toString(), "_blank", "noopener");
  });
  $("#copy-id", main).addEventListener("click", () => copy(lic.install_id || ""));
  $("#activate", main).addEventListener("click", activateModal);
  $("#pay-retry", main)?.addEventListener("click", () => act(() => api("POST", "/api/license/refresh"), null));
  $("#refresh", main).addEventListener("click", () => act(() => api("POST", "/api/license/refresh"), t("saved")));
}

// ---------- Settings ----------

/** Friendly, de-duplicated browser names for display (process names -> product). */
const BROWSER_NAMES = {
  chrome: "Chrome", "google chrome": "Chrome", msedge: "Edge", "microsoft edge": "Edge",
  brave: "Brave", "brave browser": "Brave", opera: "Opera", opera_gx: "Opera GX", operagx: "Opera GX",
  vivaldi: "Vivaldi", chromium: "Chromium", "chromium-browser": "Chromium", yandex: "Yandex", arc: "Arc",
  firefox: "Firefox", "firefox-esr": "Firefox", librewolf: "LibreWolf", waterfox: "Waterfox", zen: "Zen", floorp: "Floorp",
  safari: "Safari", tor: "Tor Browser", torbrowser: "Tor Browser", "tor browser": "Tor Browser",
  palemoon: "Pale Moon", basilisk: "Basilisk", seamonkey: "SeaMonkey", iexplore: "Internet Explorer", msie: "Internet Explorer",
  qutebrowser: "qutebrowser", falkon: "Falkon", konqueror: "Konqueror", epiphany: "GNOME Web", "gnome-web": "GNOME Web",
  midori: "Midori", otter: "Otter", "otter-browser": "Otter", maxthon: "Maxthon", ucbrowser: "UC Browser",
  "360se": "360 Browser", "360chrome": "360 Browser", duckduckgo: "DuckDuckGo", min: "Min",
};
function prettyBrowsers(names) {
  return [...new Set(names.map((n) => BROWSER_NAMES[n]).filter(Boolean))];
}

// Backups are shown as a code, not readable JSON: the text would otherwise
// list every site of the hidden lists (adult, gambling...), which can itself
// be a trigger. Restoring still goes through the agent's lock rules.
const BACKUP_PREFIX = "HTLCG1:";
function backupCode(config) {
  const bytes = new TextEncoder().encode(JSON.stringify(config));
  let bin = "";
  for (const b of bytes) bin += String.fromCharCode(b);
  return BACKUP_PREFIX + btoa(bin);
}
function readBackup(text) {
  const s = text.trim();
  if (!s.startsWith(BACKUP_PREFIX)) return JSON.parse(s); // older plain-text backups
  const bin = atob(s.slice(BACKUP_PREFIX.length).replace(/\s+/g, ""));
  return JSON.parse(new TextDecoder().decode(Uint8Array.from(bin, (c) => c.charCodeAt(0))));
}

export function renderSettings(main) {
  const st = cfg().settings;
  const browsers = S.status.browsers;
  const running = browsers.running || [];
  // While protected, a ticked protection switch can't be unticked (the agent
  // refuses it too); show it greyed out instead of failing on Save.
  const prot = cfg().protection.level !== "off" || (S.status.locked_list_ids || []).length > 0;
  const keep = (on) => (on ? `checked ${prot ? "disabled" : ""}` : "");
  main.innerHTML = `
    <div class="page-head"><h1>${t("settings_title")}</h1></div>
    <div class="stack">
      <section class="card stack">
        <label>${t("language")} ${tip("language")}<select id="s-lang">${languageOptions(st.language)}</select></label>
        <label>${t("goal")} ${tip("goal")}<input type="text" id="s-goal" maxlength="140" value="${esc(st.goal)}" placeholder="${esc(t("goal_placeholder"))}"></label>
      </section>

      <section class="card stack">
        <h2>${t("browsers_title")}</h2>
        ${prot ? `<p class="hint">🔒 ${t("err_enforcement_locked")}</p>` : ""}
        <label class="check"><input type="checkbox" id="s-unsup" ${keep(st.block_unsupported_browsers)}><span>${t("block_unsupported")} ${tip("unsupported_browsers")}<br><span class="hint">${t("block_unsupported_help")}</span></span></label>
        <label class="check"><input type="checkbox" id="s-reqext" ${keep(st.require_browser_extension)}><span>${t("guard_browsers")} ${tip("guard_browsers")}<br><span class="hint">${t("guard_browsers_help")}</span></span></label>
        <div class="stack"><strong>${t("running_browsers")} ${tip("running_browsers")}</strong>
          ${running.length === 0 ? `<p class="muted">${t("no_browser_running")}</p>` : `<div class="list">${running.map((b) => `
            <div class="row between"><span>${esc(b.name)}</span><span class="badge ${b.protected ? "ok" : b.closes ? "danger" : "warn"}">${t(b.protected ? "br_protected" : b.closes ? "br_closes" : "br_unprotected")}</span></div>`).join("")}</div>`}
        </div>
        <label class="check"><input type="checkbox" id="s-safe" ${keep(st.force_safe_search)}><span>${t("safe_search")} ${tip("safe_search")}<br><span class="hint">${t("safe_search_help")}</span></span></label>
        <label class="check"><input type="checkbox" id="s-private" ${keep(st.block_private_windows)}><span>${t("block_private")} ${tip("private_windows")}<br><span class="hint">${t("block_private_help")}</span></span></label>
        <label class="check"><input type="checkbox" id="s-managed" ${keep(st.managed_extension)}><span>${t("managed_ext")} ${tip("managed_ext")}<br><span class="hint">${t("managed_ext_help")}</span></span></label>
        ${st.managed_extension && !browsers.managed?.ready ? `<p class="banner warn">${t("managed_ext_needs_id")}</p>` : ""}
        <details><summary>${t("browser_lists")} ${tip("browser_lists")}</summary>
          <div class="stack">
            <div><strong>${t("supported_browsers")}</strong><div class="chips">${prettyBrowsers(browsers.supported).map((b) => `<span class="badge ok">${esc(b)}</span>`).join("")}</div></div>
            <div><strong>${t("unsupported_browsers")}</strong><div class="chips">${prettyBrowsers(browsers.unsupported).map((b) => `<span class="badge">${esc(b)}</span>`).join("")}</div></div>
          </div>
        </details>
      </section>

      <section class="card stack">
        <h2>${t("filters")} ${tip("filters")}</h2>
        ${FILTERS.map((f) => `<label class="check"><input type="checkbox" data-filter="${f}" ${keep(st.distraction_filters.includes(f))}>${t(`f_${f}`)}</label>`).join("")}
      </section>

      <div><button class="btn primary" id="s-save">${t("save")}</button></div>

      <details class="card stack"><summary>${t("backup_title")} ${tip("backup")}</summary>
        <p class="hint">${t("backup_help")}</p>
        <textarea id="s-backup" spellcheck="false" readonly>${esc(backupCode(cfg()))}</textarea>
        <div class="row"><button class="btn" id="s-copy">${t("backup_copy")}</button><button class="btn" id="s-import">${t("backup_import")}</button></div>
      </details>

      <section class="card stack">
        <h2>${t("updates_title")} ${tip("updates")}</h2>
        <p>${t("current_version", { v: esc(S.status.version) })}</p>
        ${S.status.update?.available
          ? `<p><strong>${t("update_available", { v: esc(S.status.update.available.version) })}</strong></p>
             <div><button class="btn primary" id="u-now">${t("update_now")}</button></div>`
          : `<p class="muted">${t("up_to_date")}</p>
             <div><button class="btn" id="u-check">${t("check_updates")}</button></div>`}
        <p class="hint">${t("update_keeps_settings")}</p>
      </section>

      <section class="card stack"><h2>${t("extension_title")} ${tip("extension")}</h2><p class="muted">${t("extension_help")}</p></section>
      <p class="muted">HTL Content Guard PC v${esc(S.status.version)}</p>
    </div>`;
  $("#s-save", main).addEventListener("click", () => saveConfig((c) => {
    c.settings.language = $("#s-lang", main).value;
    c.settings.goal = $("#s-goal", main).value.trim();
    // Two separate switches: closing browsers that can't be filtered at all
    // (Tor…) is safe for everyone; closing supported browsers whose extension
    // isn't working also hits browsers the extension can't be installed in yet.
    c.settings.block_unsupported_browsers = $("#s-unsup", main).checked;
    const guard = $("#s-reqext", main).checked;
    c.settings.require_browser_extension = guard;
    c.settings.force_safe_search = $("#s-safe", main).checked;
    // Guarding browsers needs the extension in them: install it automatically.
    c.settings.managed_extension = $("#s-managed", main).checked || guard;
    c.settings.block_private_windows = $("#s-private", main).checked;
    c.settings.distraction_filters = $$("[data-filter]", main).filter((x) => x.checked).map((x) => x.dataset.filter);
  }));
  $("#u-now", main)?.addEventListener("click", () => import("/app.js").then((m) => m.startUpdate()));
  $("#u-check", main)?.addEventListener("click", () =>
    act(() => api("POST", "/api/license/refresh"), null).then(() => {
      if (!S.status.update?.available) toast(t("up_to_date"));
    }),
  );
  $("#s-copy", main)?.addEventListener("click", () => {
    const ta = $("#s-backup", main);
    ta.select();
    navigator.clipboard?.writeText(ta.value).then(() => toast(t("saved")), () => {});
  });
  $("#s-import", main)?.addEventListener("click", () => {
    openModal(
      `<h2>${t("backup_import")}</h2><p class="muted">${t("backup_import_help")}</p>
       <textarea id="imp" spellcheck="false" placeholder="HTLCG1:..."></textarea>
       <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="imp-go">${t("backup_import")}</button></div>`,
      (m) => $("#imp-go", m).addEventListener("click", async () => {
        let parsed;
        try {
          parsed = readBackup($("#imp", m).value);
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
