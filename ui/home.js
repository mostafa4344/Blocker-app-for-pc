// Home: status, quick start, active sessions and statistics.
import { S, $, $$, t, num, esc, api, act, cfg, fmtTime, fmtDur, openModal, challengeModal, toast, errMsg } from "/core.js";
import { tip } from "/help.js";

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

function lockBody(kind) {
  switch (kind) {
    case "random_text": return { type: "random_text", length: 200 };
    case "timer": return { type: "timer" };
    case "restart": return { type: "restart" };
    case "partner": return { type: "partner" };
    default: return { type: "none" };
  }
}

async function startSession() {
  const q = S.quick;
  const minutes = Math.round(Number(q.minutes));
  if (q.listIds.length === 0) return toast(t("pick_at_least_one"), true);
  if (!(minutes > 0)) return;
  if (q.lock !== "none") {
    const end = fmtTime(new Date(Date.now() + minutes * 60000).toISOString());
    const question = t("confirm_lock", { end });
    if (!window.confirm(question)) return;
  }
  // One session per chosen list, all with the same length and lock.
  await act(async () => {
    for (const id of q.listIds) await api("POST", "/api/sessions", { block_list_id: id, minutes, lock: lockBody(q.lock) });
  }, t("focus_started", { n: num(q.listIds.length) }));
}

async function stopSession(session) {
  const kind = session.lock.type;
  const stop = (answer) => api("POST", `/api/sessions/${session.id}/stop`, answer === undefined ? {} : { answer });
  if (kind === "none") return act(() => stop());
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
  const durations = [25, 60, 120, 240];
  const lockOptions = [["none", "lock_none"], ["timer", "lock_timer"], ["random_text", "lock_random_text"], ["restart", "lock_restart"]];
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
         </div>
       </div>
       <div class="stack"><span class="hint">${t("duration")}</span>
         <div class="chips">
           ${durations.map((d) => `<button class="chip" data-min="${d}" aria-pressed="${!S.quick.custom && S.quick.minutes === d}">${durLabel(d)}</button>`).join("")}
           <button class="chip" data-min="custom" aria-pressed="${S.quick.custom}">…</button>
         </div>
         ${S.quick.custom ? `<label class="hint">${t("custom_minutes")}<input type="number" id="q-custom" min="1" max="10080" value="${S.quick.minutes}"></label>` : ""}
       </div>
       <label>${t("lock_type")} ${tip("lock_type")}
         <select id="q-lock">${lockOptions.map(([v, k]) => `<option value="${v}" ${S.quick.lock === v ? "selected" : ""}>${t(k)}</option>`).join("")}</select>
       </label>
       <div><button class="btn primary big" id="q-start">▶ ${t("start")}</button></div>`;

  const sessions = s.sessions.length === 0 ? "" : `
    <section class="card stack">
      <h2>${t("active_sessions")} ${tip("active_sessions")}</h2>
      <div class="list">
        ${s.sessions.map((x) => `
          <div class="item">
            <div><strong>${esc(cfgListName(x.block_list_id))}</strong>
              <div class="meta"><span data-countdown="${esc(x.ends_at)}"></span> ${t("ends_in")} · ${fmtTime(x.ends_at)} ${t("until")}
              ${x.lock.type !== "none" ? ` · 🔒 ${t(`lock_${x.lock.type}`)}` : ""}
              ${pendingFor(x.id) ? ` · ${t("waiting_partner")}` : ""}</div></div>
            <div class="row"><button class="btn small" data-extend="${esc(x.id)}">${t("extend")}</button><button class="btn small danger" data-stop="${esc(x.id)}">${t("stop")}</button></div>
          </div>`).join("")}
      </div>
    </section>`;

  main.innerHTML = `
    <div class="stack">
      <div class="card hero ${active.length ? "active" : ""}">
        <span class="dot" aria-hidden="true"></span>
        <div class="stack"><h2>${active.length ? t("blocking_now") : t("nothing_blocked")} ${tip("home_status")}</h2>${active.map(activeLine).join("")}</div>
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
  $("#q-lock", main)?.addEventListener("change", (e) => (S.quick.lock = e.target.value));
  $("#q-custom", main)?.addEventListener("input", (e) => (S.quick.minutes = Number(e.target.value)));
  $$("[data-min]", main).forEach((b) =>
    b.addEventListener("click", () => {
      if (b.dataset.min === "custom") S.quick.custom = true;
      else Object.assign(S.quick, { custom: false, minutes: Number(b.dataset.min) });
      S.render();
    }),
  );
  $("#q-start", main)?.addEventListener("click", startSession);
  $$("[data-stop]", main).forEach((b) => b.addEventListener("click", () => stopSession(s.sessions.find((x) => x.id === b.dataset.stop))));
  $$("[data-extend]", main).forEach((b) => b.addEventListener("click", () => act(() => api("POST", `/api/sessions/${b.dataset.extend}/extend`, { minutes: 15 }))));
}

function cfgListName(id) {
  return cfg().block_lists.find((b) => b.id === id)?.name ?? "?";
}
