// Home: status, quick start, active sessions and statistics.
import { S, $, $$, t, num, esc, api, act, cfg, fmtTime, fmtDur, openModal, challengeModal, toast, errMsg } from "/core.js";
import { tip } from "/help.js";
import { pickerHTML, mountPicker, ensureTargetList } from "/picker.js";

function banners() {
  const s = S.status;
  const out = [];
  if (!s.hosts.ok) out.push(`<div class="banner danger">${t("hosts_error")}<br><small>${esc(s.hosts.error)}</small></div>`);
  const ext = Object.values(s.extension_seen_seconds_ago || {});
  if (!ext.some((v) => v < 120)) out.push(`<div class="banner warn">${t("ext_missing")}</div>`);
  if (Math.abs(s.clock_skew_seconds) > 120) out.push(`<div class="banner info">${t("clock_changed")}</div>`);
  if (s.practice_mode) {
    out.push(`<div class="banner info row between"><span>${t("practice_mode")}</span><button class="btn small" id="end-practice">${t("end_practice")}</button></div>`);
  }
  return out.join("");
}

const reasonLabel = (a) => ({ session: t("reason_session"), schedule: t("reason_schedule"), limit: t("reason_limit"), always: t("reason_always") })[a.reason];

function activeLine(a) {
  let until = "";
  if (a.until) until = `<span data-countdown="${esc(a.until)}"></span> ${t("ends_in")} · ${fmtTime(a.until)} ${t("until")}`;
  else if (a.reason === "limit") until = t("until_midnight");
  else if (a.until_local) until = `${fmtTime(a.until_local)} ${t("until")}`;
  return `<div class="row"><strong>${esc(a.block_list_name)}</strong><span class="badge">${reasonLabel(a)}</span>${
    a.locked ? `<span class="badge lock">🔒 ${t("locked")}</span>` : ""
  }<span class="muted">${until}</span></div>`;
}

/** Requests still in progress (to unlock, lower protection or uninstall). */
const ACTIVE_REQUESTS = ["needs_challenge", "pending", "approved"];

function unlockRequestLine(r) {
  const badge = r.status === "approved" ? "ok" : "";
  let when = "";
  if (r.status === "pending" && r.available_at) when = `${t("available_at")}: ${fmtTime(r.available_at)}`;
  return `<div class="item">
      <div><div class="row"><strong>${t(`kind_${r.kind.type}`)}</strong><span class="badge ${badge}">${t(`st_${r.status}`)}</span></div>
        ${when ? `<div class="meta">${esc(when)}</div>` : ""}</div>
      <button class="btn small" data-cancel-request="${esc(r.id)}">${t("cancel")}</button>
    </div>`;
}

function lockBody(kind) {
  switch (kind) {
    case "random_text": return { type: "random_text", length: 200 };
    case "timer": return { type: "timer" };
    case "restart": return { type: "restart" };
    case "partner": return { type: "partner" };
    case "window": return { type: "window", start_minute: hhmm(S.quick.winFrom), end_minute: hhmm(S.quick.winTo) };
    default: return { type: "none" };
  }
}

const hhmm = (v) => {
  const [h, m] = String(v || "0:0").split(":").map(Number);
  return (h || 0) * 60 + (m || 0);
};
/** A Date as the value of an <input type="datetime-local">. */
const toLocalInput = (d) => {
  const z = (n) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${z(d.getMonth() + 1)}-${z(d.getDate())}T${z(d.getHours())}:${z(d.getMinutes())}`;
};
const nextHour = (fromMs, hours) => {
  const d = new Date(fromMs + hours * 3600_000);
  d.setMinutes(0, 0, 0);
  return d;
};

/** Start and end of the session the form describes (Date objects). */
function sessionTimes() {
  const q = S.quick;
  const start = q.later && q.startAt ? new Date(q.startAt) : new Date();
  const end = q.mode === "until"
    ? new Date(q.untilAt)
    : new Date(start.getTime() + Math.round(Number(q.minutes)) * 60_000);
  return { start, end };
}

/** One plain sentence: what, when, and how it can be stopped. */
function summaryText(lists) {
  const q = S.quick;
  const names = q.listIds.map((id) => lists.find((l) => l.id === id)?.name).filter(Boolean);
  if (!names.length) return t("pick_at_least_one");
  const what = names.length > 3 ? `${names.slice(0, 3).join(", ")} +${num(names.length - 3)}` : names.join(", ");
  const { start, end } = sessionTimes();
  if (!(end > start)) return t("err_end_before_start");
  const when = t("summary_when", {
    start: q.later ? fmtTime(start.toISOString()) : t("start_now_word"),
    end: fmtTime(end.toISOString()),
  });
  const how = q.lock === "window"
    ? t("lock_window_detail", { from: num(q.winFrom), to: num(q.winTo) })
    : t(`lock_${q.lock}`);
  return `${t("summary_what", { what })} ${when} ${how}${t("sentence_end")}`;
}

async function startSession() {
  const q = S.quick;
  if (q.listIds.length === 0) return toast(t("pick_at_least_one"), true);
  const { start, end } = sessionTimes();
  if (!(end > start) || Number.isNaN(end.getTime())) return toast(t("err_end_before_start"), true);
  if (q.lock === "window" && hhmm(q.winFrom) === hhmm(q.winTo)) return toast(t("err_window_empty"), true);
  if (q.lock !== "none") {
    if (!window.confirm(t("confirm_lock", { end: fmtTime(end.toISOString()) }))) return;
  }
  // "For N minutes from now" goes by the agent's trusted clock; set times are sent as times.
  const timed = q.later || q.mode === "until";
  const body = (id) => timed
    ? { block_list_id: id, starts_at: start.toISOString(), ends_at: end.toISOString(), lock: lockBody(q.lock) }
    : { block_list_id: id, minutes: Math.round(Number(q.minutes)), lock: lockBody(q.lock) };
  // One session per chosen list, all with the same times and lock.
  await act(async () => {
    for (const id of q.listIds) await api("POST", "/api/sessions", body(id));
  }, t("focus_started", { n: num(q.listIds.length) }));
}

const minHHMM = (m) => `${String(Math.floor(m / 60)).padStart(2, "0")}:${String(m % 60).padStart(2, "0")}`;

async function stopSession(session) {
  const kind = session.lock.type;
  const stop = (answer) => api("POST", `/api/sessions/${session.id}/stop`, answer === undefined ? {} : { answer });
  if (kind === "none") return act(() => stop());
  if (kind === "window") {
    try {
      await stop();
      return act(async () => {});
    } catch {
      const from = num(minHHMM(session.lock.start_minute));
      const to = num(minHHMM(session.lock.end_minute));
      return openModal(`<p>${t("window_help", { from, to })}</p><div class="actions"><button class="btn primary" data-close>${t("close")}</button></div>`);
    }
  }
  if (kind === "timer") return openModal(`<p>${t("timer_help")}</p><div class="actions"><button class="btn primary" data-close>${t("close")}</button></div>`);
  if (kind === "restart") {
    try {
      await stop();
      return act(async () => {});
    } catch {
      return openModal(`<p>${t("restart_help")}</p><div class="actions"><button class="btn primary" data-close>${t("close")}</button></div>`);
    }
  }
  if (kind === "random_text") {
    return challengeModal(`session:${session.id}`, async (answer) => {
      try {
        await stop(answer);
        await act(async () => {});
        return true;
      } catch (e) {
        toast(errMsg(e), true);
        return false;
      }
    });
  }
  if (kind === "partner") {
    // An approved request lets the stop go through; otherwise ask the partner.
    try {
      await stop();
      return act(async () => {});
    } catch {
      return act(() => api("POST", "/api/requests", { kind: { type: "end_session", session_id: session.id } }), t("waiting_partner"));
    }
  }
}

export function renderHome(main) {
  const s = S.status;
  // Always-on lists are blocked anyway; only offer the others for a session.
  const lists = cfg().block_lists.filter((l) => !l.always_on);
  // Keep only lists that still exist; on first view suggest the usual distractions.
  S.quick.listIds = S.quick.listIds.filter((id) => lists.some((l) => l.id === id));
  if (!S.quick.touched && S.quick.listIds.length === 0) {
    const suggested = lists.filter((l) => ["social", "shorts"].includes(l.preset));
    S.quick.listIds = (suggested.length ? suggested : lists.slice(0, 1)).map((l) => l.id);
  }
  const allPicked = lists.length > 0 && lists.every((l) => S.quick.listIds.includes(l.id));
  const active = s.policy.active;
  const unlock = s.requests.filter((r) => ACTIVE_REQUESTS.includes(r.status)).reverse();
  const durations = [25, 60, 120, 240];
  const lockOptions = [["none", "lock_none"], ["timer", "lock_timer"], ["window", "lock_window"], ["random_text", "lock_random_text"], ["restart", "lock_restart"]];
  const q = S.quick;
  q.mode ??= "for";
  q.untilAt ??= toLocalInput(nextHour(Date.now(), 2));
  q.startAt ??= toLocalInput(nextHour(Date.now(), 1));
  q.winFrom ??= "20:00";
  q.winTo ??= "21:00";
  if (cfg().protection.partner) lockOptions.push(["partner", "lock_partner"]);
  const top = s.stats_today.filter((x) => x.seconds > 0).slice(0, 6);
  const blockedToday = s.stats_today.reduce((a, x) => a + x.blocked_attempts, 0);
  const maxWeek = Math.max(1, ...s.week.map((d) => d.seconds));
  const dayNames = t("day_names");
  const pendingFor = (id) => s.requests.some((r) => r.kind.type === "end_session" && r.kind.session_id === id && ["pending", "approved"].includes(r.status));
  const durLabel = (d) => (d < 60 ? `${num(d)} ${t("minutes_short")}` : `${num(d / 60)} ${t("hours_short")}`);

  const quick = lists.length === 0
    ? `<p class="muted">${t("add_preset")}</p><div><button class="btn primary" data-goto="lists">${t("nav_lists")}</button></div>`
    : `<div class="stack"><span class="hint">${t("choose_lists")}</span>
         <div class="chips" role="group" aria-label="${esc(t("choose_lists"))}">
           ${lists.map((l) => `<button class="chip" data-pick="${esc(l.id)}" aria-pressed="${S.quick.listIds.includes(l.id)}">${esc(l.name)}</button>`).join("")}
           ${lists.length > 1 ? `<button class="chip ghost" data-pick-all>${t(allPicked ? "pick_none" : "pick_all")}</button>` : ""}
           <button class="chip ghost" id="q-add-target" aria-expanded="${!!q.picking}">+ ${t("add_target")}</button>
         </div>
         ${q.picking ? pickerHTML() : ""}
       </div>
       <div class="stack"><span class="hint">${t("duration")}</span>
         <div class="chips">
           ${durations.map((d) => `<button class="chip" data-min="${d}" aria-pressed="${q.mode === "for" && !q.custom && q.minutes === d}">${durLabel(d)}</button>`).join("")}
           <button class="chip" data-min="custom" aria-pressed="${q.mode === "for" && q.custom}">…</button>
           <button class="chip" data-min="until" aria-pressed="${q.mode === "until"}">${t("until_time")}</button>
         </div>
         ${q.mode === "for" && q.custom ? `<label class="hint">${t("custom_minutes")}<input type="number" id="q-custom" min="1" max="10080" value="${q.minutes}"></label>` : ""}
         ${q.mode === "until" ? `<label class="hint">${t("until_label")}<input type="datetime-local" id="q-until" value="${esc(q.untilAt)}"></label>` : ""}
       </div>
       <div class="stack"><span class="hint">${t("start_label")}</span>
         <div class="chips">
           <button class="chip" data-later="0" aria-pressed="${!q.later}">${t("start_now")}</button>
           <button class="chip" data-later="1" aria-pressed="${!!q.later}">${t("start_later")}</button>
         </div>
         ${q.later ? `<label class="hint">${t("starts_at_label")}<input type="datetime-local" id="q-start-at" value="${esc(q.startAt)}"></label>` : ""}
       </div>
       <label>${t("lock_type")} ${tip("lock_type")}
         <select id="q-lock">${lockOptions.map(([v, k]) => `<option value="${v}" ${q.lock === v ? "selected" : ""}>${t(k)}</option>`).join("")}</select>
       </label>
       ${q.lock === "window" ? `<div class="grid two"><label class="hint">${t("window_from")}<input type="time" id="q-wfrom" value="${esc(q.winFrom)}"></label>
         <label class="hint">${t("window_to")}<input type="time" id="q-wto" value="${esc(q.winTo)}"></label></div>` : ""}
       <p class="summary" id="q-summary">${esc(summaryText(lists))}</p>
       <div><button class="btn primary big" id="q-start">▶ ${t("start")}</button></div>`;

  const sessions = s.sessions.length === 0 ? "" : `
    <section class="card stack">
      <h2>${t("active_sessions")} ${tip("active_sessions")}</h2>
      <div class="list">
        ${s.sessions.map((x) => `
          <div class="item">
            <div><strong>${esc(cfgListName(x.block_list_id))}</strong>
              <div class="meta">${new Date(x.started_at) > new Date()
                ? `${t("starts_at_short", { time: fmtTime(x.started_at) })} · ${fmtTime(x.ends_at)} ${t("until")}`
                : `<span data-countdown="${esc(x.ends_at)}"></span> ${t("ends_in")} · ${fmtTime(x.ends_at)} ${t("until")}`}
              ${x.lock.type === "window" ? ` · 🔒 ${t("lock_window_detail", { from: num(minHHMM(x.lock.start_minute)), to: num(minHHMM(x.lock.end_minute)) })}` : x.lock.type !== "none" ? ` · 🔒 ${t(`lock_${x.lock.type}`)}` : ""}
              ${pendingFor(x.id) ? ` · ${t("waiting_partner")}` : ""}</div></div>
            <div class="row"><button class="btn small" data-extend="${esc(x.id)}">${t("extend")}</button><button class="btn small danger" data-stop="${esc(x.id)}">${t("stop")}</button></div>
          </div>`).join("")}
      </div>
    </section>`;

  main.innerHTML = `
    <div class="stack">
      <div class="card hero-split">
        <div class="hero ${active.length ? "active" : ""}">
          <span class="dot" aria-hidden="true"></span>
          <div class="stack"><h2>${active.length ? t("blocking_now") : t("nothing_blocked")} ${tip("home_status")}</h2>${active.map(activeLine).join("")}</div>
        </div>
        <div class="stack unlock-col">
          <div class="row between"><h2>${t("unlock_requests")} ${tip("requests")}</h2>
            <button class="btn small" data-goto="protection">${t("see_details")}</button></div>
          ${unlock.length
            ? `<div class="list">${unlock.map(unlockRequestLine).join("")}</div>`
            : `<p class="muted">${t("no_unlock_requests")}</p>`}
        </div>
      </div>
      ${banners()}
      <div class="grid two">
        <section class="card stack"><h2>${t("quick_start")} ${tip("quick_start")}</h2>${quick}</section>
        <section class="card stack">
          <div class="row between"><h2>${t("today")} ${tip("today_stats")}</h2><span class="badge">${t("blocked_attempts")}: ${num(blockedToday)}</span></div>
          <h3>${t("top_usage")}</h3>
          ${top.length === 0 ? `<p class="muted">${t("no_data")}</p>` : `<div class="list">${top.map((x) => `<div class="row between"><span>${esc(x.key.replace(/^(site|app):/, ""))}</span><span class="muted">${fmtDur(x.seconds)}</span></div>`).join("")}</div>`}
          <h3>${t("this_week")}</h3>
          <div class="bars">${s.week.map((d) => `<div class="bar" title="${fmtDur(d.seconds)}"><div class="fill" data-h="${Math.round((d.seconds / maxWeek) * 100)}"></div><small>${dayNames[(new Date(d.day + "T12:00:00").getDay() + 6) % 7]}</small></div>`).join("")}</div>
        </section>
      </div>
      ${sessions}
    </div>`;

  $$(".bar .fill", main).forEach((el) => (el.style.height = `${Math.max(2, Number(el.dataset.h))}%`));
  $("#end-practice", main)?.addEventListener("click", () => act(() => api("POST", "/api/practice/end")));
  // Cancelling an unlock request only keeps things locked, so no confirmation is needed.
  $$("[data-cancel-request]", main).forEach((b) => b.addEventListener("click", () =>
    act(() => api("POST", `/api/requests/${b.dataset.cancelRequest}/cancel`), t("request_cancelled"))));
  $$("[data-goto]", main).forEach((b) => b.addEventListener("click", () => { S.page = b.dataset.goto; S.render(); }));
  $$("[data-pick]", main).forEach((b) => b.addEventListener("click", () => {
    const id = b.dataset.pick;
    const q = S.quick;
    q.touched = true;
    q.listIds = q.listIds.includes(id) ? q.listIds.filter((x) => x !== id) : [...q.listIds, id];
    S.render();
  }));
  $("[data-pick-all]", main)?.addEventListener("click", () => {
    S.quick.touched = true;
    S.quick.listIds = allPicked ? [] : lists.map((l) => l.id);
    S.render();
  });
  const summary = () => { const el = $("#q-summary", main); if (el) el.textContent = summaryText(lists); };
  $("#q-lock", main)?.addEventListener("change", (e) => { S.quick.lock = e.target.value; S.render(); });
  $("#q-custom", main)?.addEventListener("input", (e) => { S.quick.minutes = Number(e.target.value); summary(); });
  $("#q-until", main)?.addEventListener("input", (e) => { S.quick.untilAt = e.target.value; summary(); });
  $("#q-start-at", main)?.addEventListener("input", (e) => { S.quick.startAt = e.target.value; summary(); });
  $("#q-wfrom", main)?.addEventListener("input", (e) => { S.quick.winFrom = e.target.value; summary(); });
  $("#q-wto", main)?.addEventListener("input", (e) => { S.quick.winTo = e.target.value; summary(); });
  $$("[data-later]", main).forEach((b) => b.addEventListener("click", () => { S.quick.later = b.dataset.later === "1"; S.render(); }));
  $$("[data-min]", main).forEach((b) =>
    b.addEventListener("click", () => {
      if (b.dataset.min === "custom") Object.assign(S.quick, { mode: "for", custom: true });
      else if (b.dataset.min === "until") S.quick.mode = "until";
      else Object.assign(S.quick, { mode: "for", custom: false, minutes: Number(b.dataset.min) });
      S.render();
    }),
  );
  $("#q-add-target", main)?.addEventListener("click", () => { S.quick.picking = !S.quick.picking; S.render(); });
  const pickerRoot = $(".picker", main);
  if (pickerRoot) {
    mountPicker(pickerRoot, async (target) => {
      const id = await ensureTargetList(target);
      if (!id) return;
      Object.assign(S.quick, { picking: false, touched: true });
      if (!S.quick.listIds.includes(id)) S.quick.listIds.push(id);
      S.render();
    });
  }
  $("#q-start", main)?.addEventListener("click", startSession);
  $$("[data-stop]", main).forEach((b) => b.addEventListener("click", () => stopSession(s.sessions.find((x) => x.id === b.dataset.stop))));
  $$("[data-extend]", main).forEach((b) => b.addEventListener("click", () => act(() => api("POST", `/api/sessions/${b.dataset.extend}/extend`, { minutes: 15 }))));
}

function cfgListName(id) {
  return cfg().block_lists.find((b) => b.id === id)?.name ?? "?";
}
