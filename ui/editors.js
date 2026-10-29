// Block lists, schedules and daily limits.
import { tip } from "/help.js";
import { pickerHTML, mountPicker, ensureTargetList } from "/picker.js";
import { S, $, $$, t, presetName, addPreset, num, esc, api, act, cfg, fmtDur, fmtTime, openModal, closeModal, saveConfig, toast, lines, newId, isLocked, listName, minToHHMM, hhmmToMin } from "/core.js";

const lockBadge = (on) => (on ? `<span class="badge lock">🔒 ${t("locked")}</span>` : "");
/** A schedule/limit that can only be tightened: locked itself, or enabled while protection is on. */
const guardedItem = (x) => x.locked || (x.enabled && cfg().protection.level !== "off");

// ---------- block lists ----------

/** Ready-made lists whose contents are never shown (they would read as a list of triggers). */
const SENSITIVE_PRESETS = ["adult", "gambling", "dating"];
const isSensitive = (l) => SENSITIVE_PRESETS.includes(l.preset) || (!l.preset && l.keywords.some((k) => /porn|xxx|hentai/i.test(k)));
const merge = (a, b) => [...a, ...b.filter((x) => !a.some((y) => y.toLowerCase() === x.toLowerCase()))];

/** Splits typed exceptions into websites/pages and apps ("minecraft.exe", "Discord"). */
function splitAllowed(text) {
  const sites = [];
  const apps = [];
  for (const raw of lines(text)) {
    const v = raw.replace(/^["'\[\]]+|["'\[\]]+$/g, "").trim();
    if (!v) continue;
    if (/\.exe$/i.test(v) || !/[./]/.test(v)) {
      apps.push(v);
    } else {
      const site = v.toLowerCase().replace(/^[a-z]+:\/\//, "").replace(/^www\./, "").replace(/[?#].*$/, "").replace(/\/+$/, "");
      if (site && site !== "*") sites.push(site);
    }
  }
  return { sites, apps };
}
const allowedText = (l) => [...(l.allow_sites || []), ...(l.allow_apps || [])].join("\n");

function countLine(l) {
  const parts = [["count_sites", l.sites], ["count_apps", l.apps], ["count_words", l.keywords],
    ["count_allowed", [...(l.allow_sites || []), ...(l.allow_apps || [])]]]
    .filter(([, v]) => v.length > 0)
    .map(([k, v]) => t(k, { n: num(v.length) }));
  return parts.length ? parts.join(" · ") : t("count_empty");
}

function editList(list) {
  const isNew = !list;
  const l = list ?? { id: newId(), name: "", sites: [], apps: [], keywords: [], allow_sites: [], allow_apps: [], allowlist_mode: false, always_on: false, preset: "" };
  const locked = !isNew && isLocked(l.id);
  const hidden = !isNew && isSensitive(l);
  const contents = hidden
    ? `<div class="banner info">${t("hidden_list_note", { sites: num(l.sites.length), words: num(l.keywords.length) })} ${tip("hidden_list")}</div>
       <label>${t("add_sites")} ${tip("list_sites")}<span class="hint">${t("sites_hint")}</span><textarea id="l-sites"></textarea></label>
       <label>${t("add_words")} ${tip("list_keywords")}<textarea id="l-keywords"></textarea></label>`
    : `<label>${t("sites")} ${tip("list_sites")}<span class="hint">${t("sites_hint")}</span><textarea id="l-sites">${esc(l.sites.join("\n"))}</textarea></label>
       <label>${t("apps")} ${tip("list_apps")}<span class="hint">${t("apps_hint")}</span><textarea id="l-apps">${esc(l.apps.join("\n"))}</textarea></label>
       <label>${t("keywords")} ${tip("list_keywords")}<textarea id="l-keywords">${esc(l.keywords.join("\n"))}</textarea></label>
       <label class="check"><input type="checkbox" id="l-allow" ${l.allowlist_mode ? "checked" : ""} ${locked ? "disabled" : ""}>${t("allowlist_mode")} ${tip("allowlist")}</label>`;
  const waiting = S.status.requests
    .filter((r) => r.kind.type === "add_exceptions" && r.kind.list_id === l.id && ["needs_challenge", "pending", "approved"].includes(r.status))
    .map((r) => `<p class="hint">⏳ ${esc([...r.kind.sites, ...r.kind.apps].join(", "))} — ${r.available_at && r.status === "pending" ? `${t("available_at")}: ${esc(fmtTime(r.available_at))}` : t(`st_${r.status}`)}</p>`)
    .join("");
  const allowBox = l.allowlist_mode ? "" : `
     <label>${t("allow_exceptions")} ${tip("allow_exceptions")}<span class="hint">${t(locked ? "allow_locked_note" : "allow_hint")} ${t("allow_page_note")}</span>
       <textarea id="l-allowed" placeholder="youtube.com/@khanacademy&#10;mail.google.com&#10;minecraft.exe">${esc(allowedText(l))}</textarea></label>${waiting}`;
  openModal(
    `<h2>${isNew ? t("add_list") : esc(l.name)}</h2>
     ${locked ? `<div class="banner warn">${t("list_locked_note")}</div>` : ""}
     <label>${t("name")}<input type="text" id="l-name" value="${esc(l.name)}" maxlength="60"></label>
     ${contents}
     ${allowBox}
     <label class="check"><input type="checkbox" id="l-always" ${l.always_on ? "checked" : ""} ${locked && l.always_on ? "disabled" : ""}><span>${t("always_on")} ${tip("always_on")}<span class="hint">${t("always_on_hint")}</span></span></label>
     <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="l-save">${t("save")}</button></div>`,
    (m) => {
      $("#l-save", m).addEventListener("click", async () => {
        const allowed = $("#l-allowed", m) ? splitAllowed($("#l-allowed", m).value) : { sites: l.allow_sites || [], apps: l.allow_apps || [] };
        // A locked list can't be loosened at once: new exceptions become a
        // protection request (same waiting time as other loosening); removing works now.
        const has = (list, v) => (list || []).some((x) => x.toLowerCase() === v.toLowerCase());
        const added = locked
          ? { sites: allowed.sites.filter((v) => !has(l.allow_sites, v)), apps: allowed.apps.filter((v) => !has(l.allow_apps, v)) }
          : { sites: [], apps: [] };
        const exceptions = {
          allow_sites: allowed.sites.filter((v) => !has(added.sites, v)),
          allow_apps: allowed.apps.filter((v) => !has(added.apps, v)),
        };
        const updated = hidden
          ? {
              ...l,
              name: $("#l-name", m).value.trim() || l.name,
              sites: merge(l.sites, lines($("#l-sites", m).value)),
              keywords: merge(l.keywords, lines($("#l-keywords", m).value)),
              always_on: $("#l-always", m).checked,
              ...exceptions,
            }
          : {
              ...l,
              name: $("#l-name", m).value.trim() || t("add_list"),
              sites: lines($("#l-sites", m).value),
              apps: lines($("#l-apps", m).value),
              keywords: lines($("#l-keywords", m).value),
              allowlist_mode: $("#l-allow", m).checked,
              always_on: $("#l-always", m).checked,
              ...exceptions,
            };
        const needsRequest = added.sites.length + added.apps.length > 0;
        const ok = await saveConfig((c) => {
          const i = c.block_lists.findIndex((b) => b.id === l.id);
          if (i >= 0) c.block_lists[i] = updated;
          else c.block_lists.push(updated);
        }, needsRequest ? null : undefined);
        if (ok !== true) return;
        if (needsRequest) {
          const done = await act(
            () => api("POST", "/api/requests", { kind: { type: "add_exceptions", list_id: l.id, sites: added.sites, apps: added.apps } }),
            t("exceptions_requested"),
          );
          if (!done) return;
        }
        closeModal();
      });
    },
  );
}

function presetPicker() {
  const presets = S.status.presets;
  openModal(
    `<h2>${t("add_preset")} ${tip("presets")}</h2>
     <div class="preset-grid">${presets.map((p) => `<button class="preset" data-preset="${esc(p.id)}"><span class="icon">${p.icon}</span><strong>${esc(presetName(p))}</strong><small>${num(p.sites.length)} · ${num(p.apps.length)}</small></button>`).join("")}</div>
     <div class="actions"><button class="btn" data-close>${t("close")}</button></div>`,
    (m) => $$("[data-preset]", m).forEach((b) => b.addEventListener("click", async () => {
      if (await act(() => addPreset(b.dataset.preset), t("saved"))) closeModal();
    })),
  );
}

export function renderLists(main) {
  const lists = cfg().block_lists;
  main.innerHTML = `
    <div class="page-head"><div><h1>${t("lists_title")} ${tip("lists_overview")}</h1><p class="muted">${t("lists_intro")}</p></div>
      <div class="row"><button class="btn" id="add-preset">${t("add_preset")}</button><button class="btn primary" id="add-list">+ ${t("add_list")}</button></div></div>
    <div class="list">
      ${lists.length === 0 ? `<p class="muted">${t("no_data")}</p>` : lists.map((l) => `
        <div class="item">
          <div><div class="row"><strong>${esc(l.name)}</strong>${lockBadge(isLocked(l.id))}${l.always_on ? `<span class="badge ok">${t("badge_always")}</span>` : ""}${l.allowlist_mode ? `<span class="badge">allowlist</span>` : ""}</div>
            <div class="meta">${countLine(l)}</div></div>
          <div class="row"><button class="btn small" data-edit="${esc(l.id)}">${t("edit")}</button>${isLocked(l.id) ? "" : `<button class="btn small danger" data-del="${esc(l.id)}">${t("delete")}</button>`}</div>
        </div>`).join("")}
    </div>`;
  $("#add-list", main).addEventListener("click", () => editList(null));
  $("#add-preset", main).addEventListener("click", presetPicker);
  $$("[data-edit]", main).forEach((b) => b.addEventListener("click", () => editList(lists.find((l) => l.id === b.dataset.edit))));
  $$("[data-del]", main).forEach((b) => b.addEventListener("click", () => {
    if (!window.confirm(t("confirm_delete"))) return;
    const id = b.dataset.del;
    saveConfig((c) => {
      c.block_lists = c.block_lists.filter((x) => x.id !== id);
      c.schedules = c.schedules.filter((x) => x.block_list_id !== id);
      c.limits = c.limits.filter((x) => x.block_list_id !== id);
    });
  }));
}

// ---------- schedules ----------

/** "+ Site or app" under a list choice: picks one site/app and selects its list. */
function targetPickerRow(selectId, disabled) {
  return disabled ? "" : `<div class="stack"><button type="button" class="btn small" data-open-picker="${selectId}">+ ${t("add_target")}</button>
      <div class="picker-slot" data-for="${selectId}"></div></div>`;
}
function wireTargetPicker(m) {
  $$("[data-open-picker]", m).forEach((b) => b.addEventListener("click", () => {
    const selectId = b.dataset.openPicker;
    const slot = $(`.picker-slot[data-for="${selectId}"]`, m);
    if (slot.innerHTML) {
      slot.innerHTML = "";
      return;
    }
    slot.innerHTML = pickerHTML();
    mountPicker(slot, async (target) => {
      const id = await ensureTargetList(target);
      if (!id) return;
      const sel = $(`#${selectId}`, m);
      sel.innerHTML = listOptions(id);
      sel.value = id;
      slot.innerHTML = "";
    });
    $(".picker-q", slot)?.focus();
  }));
}

function listOptions(selected) {
  // Always-on lists are blocked 24/7 already, so they are not offered (except
  // when an existing item still points at one).
  return cfg().block_lists.filter((l) => !l.always_on || l.id === selected).map((l) => `<option value="${esc(l.id)}" ${l.id === selected ? "selected" : ""}>${esc(l.name)}</option>`).join("");
}

/** Working days: Sunday-Thursday where that is the custom (Bangladesh, Gulf), else Monday-Friday. */
const workDays = () => (["bn", "ar"].includes(cfg().settings.language) ? [0, 1, 2, 3, 6] : [0, 1, 2, 3, 4]);
const EVERY_DAY = [0, 1, 2, 3, 4, 5, 6];
/** One-tap starting points for a new schedule (days and times stay editable). */
const TEMPLATES = [
  { id: "work", days: workDays, start: 9 * 60, end: 17 * 60 },
  { id: "school", days: workDays, start: 8 * 60, end: 15 * 60 },
  { id: "study", days: () => EVERY_DAY, start: 18 * 60, end: 21 * 60 },
  { id: "bedtime", days: () => EVERY_DAY, start: 23 * 60, end: 6 * 60 },
  { id: "allday", days: () => EVERY_DAY, start: 0, end: 0 },
];

/** Time range text; equal start and end means the whole day. */
const rangeText = (s) => (s.start_minute === s.end_minute ? t("all_day") : `${num(minToHHMM(s.start_minute))}–${num(minToHHMM(s.end_minute))}`);

/** Pieces of a schedule on the week grid: [day, fromMinute, toMinute]; overnight windows continue next day. */
export function weekSegments(s) {
  const out = [];
  for (const d of s.days) {
    if (s.start_minute === s.end_minute) out.push([d, 0, 1440]);
    else if (s.start_minute < s.end_minute) out.push([d, s.start_minute, s.end_minute]);
    else {
      out.push([d, s.start_minute, 1440]);
      if (s.end_minute > 0) out.push([(d + 1) % 7, 0, s.end_minute]);
    }
  }
  return out;
}

function weekView(items) {
  const active = items.filter((s) => s.enabled && s.days.length);
  if (!active.length) return "";
  const lists = [...new Set(active.map((s) => s.block_list_id))];
  const color = (id) => `var(--wk-${lists.indexOf(id) % 6})`;
  const now = new Date();
  const today = (now.getDay() + 6) % 7;
  const nowPct = ((now.getHours() * 60 + now.getMinutes()) / 1440) * 100;
  const names = t("day_names");
  const rows = names.map((n, d) => {
    const segs = active.flatMap((s) => weekSegments(s).filter((x) => x[0] === d).map((x) => {
      const label = `${listName(s.block_list_id)} · ${num(minToHHMM(x[1]))}–${num(minToHHMM(x[2] % 1440))}`;
      return `<span class="wk-seg" data-x="${(x[1] / 1440) * 100}" data-wd="${((x[2] - x[1]) / 1440) * 100}" data-c="${color(s.block_list_id)}" title="${esc(label)}"></span>`;
    })).join("");
    return `<div class="wk-row${d === today ? " today" : ""}"><span class="wk-day">${esc(n)}</span><div class="wk-track">${segs}${d === today ? `<span class="wk-now" data-x="${nowPct}"></span>` : ""}</div></div>`;
  }).join("");
  const hours = [0, 6, 12, 18, 24].map((h) => `<span data-x="${(h / 24) * 100}">${num(String(h).padStart(2, "0"))}</span>`).join("");
  const legend = lists.map((id) => `<span class="wk-key"><i data-c="${color(id)}"></i>${esc(listName(id))}</span>`).join("");
  return `<div class="card week-card"><div class="row"><strong>${t("week_view")}</strong>${tip("week_view")}</div>
    <div class="wk-hours"><span class="wk-day"></span><div>${hours}</div></div>${rows}
    <div class="wk-legend">${legend}</div></div>`;
}

/** Positions and colors come from data-* (the page's security policy blocks inline styles). */
function applyWeek(root) {
  $$(".week-card [data-x]", root).forEach((el) => (el.style.insetInlineStart = `${el.dataset.x}%`));
  $$(".week-card [data-wd]", root).forEach((el) => (el.style.width = `${el.dataset.wd}%`));
  $$(".week-card [data-c]", root).forEach((el) => (el.style.background = el.dataset.c));
}

function daysSummary(days) {
  if (days.length === 7) return t("weekdays_all");
  const names = t("day_names");
  return [...days].sort().map((d) => names[d]).join(", ");
}

function editSchedule(sched) {
  const isNew = !sched;
  const s = sched ?? { id: newId(), block_list_id: cfg().block_lists.find((b) => !b.always_on)?.id ?? "", days: [0, 1, 2, 3, 4], start_minute: 540, end_minute: 1020, enabled: true, locked: false };
  const names = t("day_names");
  openModal(
    `<h2>${isNew ? t("add_schedule") : t("schedules_title")}</h2>
     ${!isNew && guardedItem(s) ? `<div class="banner warn">${t("lock_schedule")}</div>` : ""}
     <label>${t("choose_list")}<select id="s-list" ${!isNew && guardedItem(s) ? "disabled" : ""}>${listOptions(s.block_list_id)}</select></label>
     ${targetPickerRow("s-list", !isNew && guardedItem(s))}
     ${isNew ? `<div class="stack"><span class="hint">${t("templates")}</span><div class="chips">${TEMPLATES.map((x) => `<button type="button" class="chip ghost" data-template="${x.id}">${t(`tpl_${x.id}`)}</button>`).join("")}</div></div>` : ""}
     <div class="stack"><span class="hint">${t("days")}</span><div class="chips">${names.map((n, i) => `<button class="chip" data-day="${i}" aria-pressed="${s.days.includes(i)}">${n}</button>`).join("")}</div></div>
     <div class="grid two"><label>${t("start_time")} ${tip("schedule_times")}<input type="time" id="s-start" value="${minToHHMM(s.start_minute)}"></label>
       <label>${t("end_time")}<input type="time" id="s-end" value="${minToHHMM(s.end_minute)}"></label></div>
     <label class="check"><input type="checkbox" id="s-locked" ${s.locked ? "checked" : ""}>${t("lock_schedule")} ${tip("schedule_lock")}</label>
     <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="s-save">${t("save")}</button></div>`,
    (m) => {
      wireTargetPicker(m);
      const days = new Set(s.days);
      $$("[data-day]", m).forEach((b) => b.addEventListener("click", () => {
        const d = Number(b.dataset.day);
        days.has(d) ? days.delete(d) : days.add(d);
        b.setAttribute("aria-pressed", days.has(d));
      }));
      $$("[data-template]", m).forEach((b) => b.addEventListener("click", () => {
        const x = TEMPLATES.find((y) => y.id === b.dataset.template);
        days.clear();
        x.days().forEach((d) => days.add(d));
        $$("[data-day]", m).forEach((c) => c.setAttribute("aria-pressed", days.has(Number(c.dataset.day))));
        $("#s-start", m).value = minToHHMM(x.start);
        $("#s-end", m).value = minToHHMM(x.end);
        $$("[data-template]", m).forEach((c) => c.setAttribute("aria-pressed", c === b));
      }));
      $("#s-save", m).addEventListener("click", async () => {
        if (!$("#s-list", m).value) return toast(t("pick_at_least_one"), true);
        if (days.size === 0) return toast(t("err_pick_days"), true);
        const updated = {
          ...s,
          block_list_id: $("#s-list", m).value,
          days: [...days].sort(),
          start_minute: hhmmToMin($("#s-start", m).value),
          end_minute: hhmmToMin($("#s-end", m).value),
          locked: $("#s-locked", m).checked,
        };
        const ok = await saveConfig((c) => {
          const i = c.schedules.findIndex((x) => x.id === s.id);
          if (i >= 0) c.schedules[i] = updated;
          else c.schedules.push(updated);
        });
        if (ok === true) closeModal();
      });
    },
  );
}

export function renderSchedules(main) {
  const items = cfg().schedules;
  main.innerHTML = `
    <div class="page-head"><div><h1>${t("schedules_title")} ${tip("schedules_overview")}</h1><p class="muted">${t("schedules_intro")}</p></div><button class="btn primary" id="add">+ ${t("add_schedule")}</button></div>
    ${weekView(items)}
    <div class="list">${items.length === 0 ? `<p class="muted">${t("no_schedules")}</p>` : items.map((s) => `
      <div class="item">
        <div><div class="row"><strong>${esc(listName(s.block_list_id))}</strong>${lockBadge(guardedItem(s))}</div>
          <div class="meta">${daysSummary(s.days)} · ${rangeText(s)}</div></div>
        <div class="row">
          <label class="check"><input type="checkbox" data-toggle="${esc(s.id)}" ${s.enabled ? "checked" : ""} ${guardedItem(s) ? "disabled" : ""}>${t("enabled")}</label>
          <button class="btn small" data-edit="${esc(s.id)}">${t("edit")}</button>
          ${guardedItem(s) ? "" : `<button class="btn small danger" data-del="${esc(s.id)}">${t("delete")}</button>`}
        </div>
      </div>`).join("")}</div>`;
  applyWeek(main);
  $("#add", main).addEventListener("click", () => editSchedule(null));
  $$("[data-edit]", main).forEach((b) => b.addEventListener("click", () => editSchedule(items.find((x) => x.id === b.dataset.edit))));
  $$("[data-toggle]", main).forEach((b) => b.addEventListener("change", () => saveConfig((c) => {
    const x = c.schedules.find((y) => y.id === b.dataset.toggle);
    if (x) x.enabled = b.checked;
  })));
  $$("[data-del]", main).forEach((b) => b.addEventListener("click", () => {
    if (window.confirm(t("confirm_delete"))) saveConfig((c) => (c.schedules = c.schedules.filter((x) => x.id !== b.dataset.del)));
  }));
}

// ---------- limits ----------

/** Today's weekday, Monday = 0 (the agent uses the same order). */
const todayIndex = () => (new Date().getDay() + 6) % 7;
/** Allowed minutes on a weekday: the per-day value when set, else the daily one. */
export const minutesOn = (l, d) => (l.day_minutes?.length === 7 ? l.day_minutes[d] : l.daily_minutes);

function editLimit(limit) {
  const isNew = !limit;
  const l = limit ?? { id: newId(), block_list_id: cfg().block_lists.find((b) => !b.always_on)?.id ?? "", daily_minutes: 30, day_minutes: [], max_opens: null, enabled: true, locked: false };
  const perDay = l.day_minutes?.length === 7;
  openModal(
    `<h2>${isNew ? t("add_limit") : t("limits_title")}</h2>
     <label>${t("choose_list")}<select id="m-list" ${!isNew && guardedItem(l) ? "disabled" : ""}>${listOptions(l.block_list_id)}</select></label>
     ${targetPickerRow("m-list", !isNew && guardedItem(l))}
     <label>${t("daily_minutes")} ${tip("limit_minutes")}<input type="number" id="m-min" min="0" max="1440" value="${l.daily_minutes}"></label>
     <label class="check"><input type="checkbox" id="m-perday" ${perDay ? "checked" : ""}>${t("limit_per_day")} ${tip("limit_days")}</label>
     <div id="m-days" class="day-minutes" ${perDay ? "" : "hidden"}>${t("day_names").map((d, i) => `
       <label>${esc(d)}<input type="number" min="0" max="1440" data-day="${i}" value="${minutesOn(l, i)}"></label>`).join("")}
       <p class="hint">${t("limit_per_day_hint")}</p>
     </div>
     <label>${t("max_opens")} ${tip("limit_opens")}<input type="number" id="m-opens" min="1" max="500" value="${l.max_opens ?? ""}"></label>
     <label class="check"><input type="checkbox" id="m-locked" ${l.locked ? "checked" : ""}>${t("lock_limit")} ${tip("limit_lock")}</label>
     <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="m-save">${t("save")}</button></div>`,
    (m) => { wireTargetPicker(m);
      const minutes = (el) => Math.min(1440, Math.max(0, Math.round(Number(el.value) || 0)));
      // Turning per-day on starts every day from the daily value; changing the
      // daily value later updates the days that still match it.
      $("#m-perday", m).addEventListener("change", (e) => {
        $("#m-days", m).hidden = !e.target.checked;
        if (e.target.checked) $$("#m-days input", m).forEach((i) => (i.value = minutes($("#m-min", m))));
      });
      let prevDaily = minutes($("#m-min", m));
      $("#m-min", m).addEventListener("input", () => {
        const v = minutes($("#m-min", m));
        $$("#m-days input", m).forEach((i) => { if (minutes(i) === prevDaily) i.value = v; });
        prevDaily = v;
      });
      $("#m-save", m).addEventListener("click", async () => {
      if (!$("#m-list", m).value) return toast(t("pick_at_least_one"), true);
      const opens = $("#m-opens", m).value.trim();
      const daily = minutes($("#m-min", m));
      const days = $$("#m-days input", m).map(minutes);
      const updated = {
        ...l,
        block_list_id: $("#m-list", m).value,
        daily_minutes: daily,
        day_minutes: $("#m-perday", m).checked && days.some((d) => d !== daily) ? days : [],
        max_opens: opens ? Math.max(1, Math.round(Number(opens))) : null,
        locked: $("#m-locked", m).checked,
      };
      const ok = await saveConfig((c) => {
        const i = c.limits.findIndex((x) => x.id === l.id);
        if (i >= 0) c.limits[i] = updated;
        else c.limits.push(updated);
      });
      if (ok === true) closeModal();
    }); },
  );
}

export function renderLimits(main) {
  const items = cfg().limits;
  const usage = S.status.usage_today;
  main.innerHTML = `
    <div class="page-head"><div><h1>${t("limits_title")} ${tip("limits_overview")}</h1><p class="muted">${t("limits_intro")}</p></div><button class="btn primary" id="add">+ ${t("add_limit")}</button></div>
    <div class="list">${items.length === 0 ? `<p class="muted">${t("no_limits")}</p>` : items.map((l) => {
      const u = usage.find((x) => x.limit_id === l.id) ?? { seconds: 0, opens: 0 };
      const today = minutesOn(l, todayIndex());
      const pct = today ? Math.min(100, Math.round((u.seconds / (today * 60)) * 100)) : 100;
      const week = l.day_minutes?.length === 7
        ? `<div class="meta">${t("day_names").map((d, i) => `${esc(d)} ${l.day_minutes[i] ? fmtDur(l.day_minutes[i] * 60) : t("day_off")}`).join(" · ")}</div>` : "";
      return `
      <div class="item">
        <div class="stack">
          <div class="row"><strong>${esc(listName(l.block_list_id))}</strong>${lockBadge(guardedItem(l))}</div>
          <div class="meta">${t("used_today")}: ${fmtDur(u.seconds)} / ${fmtDur(today * 60)}${l.max_opens ? ` · ${num(u.opens)}/${num(l.max_opens)} ${t("opens")}` : ""}</div>
          ${week}
          <div class="progress ${pct >= 100 ? "full" : ""}"><div data-w="${pct}"></div></div>
        </div>
        <div class="row">
          <label class="check"><input type="checkbox" data-toggle="${esc(l.id)}" ${l.enabled ? "checked" : ""} ${guardedItem(l) ? "disabled" : ""}>${t("enabled")}</label>
          <button class="btn small" data-edit="${esc(l.id)}">${t("edit")}</button>
          ${guardedItem(l) ? "" : `<button class="btn small danger" data-del="${esc(l.id)}">${t("delete")}</button>`}
        </div>
      </div>`;
    }).join("")}</div>`;
  $$(".progress > div", main).forEach((el) => (el.style.width = `${el.dataset.w}%`));
  $("#add", main).addEventListener("click", () => editLimit(null));
  $$("[data-edit]", main).forEach((b) => b.addEventListener("click", () => editLimit(items.find((x) => x.id === b.dataset.edit))));
  $$("[data-toggle]", main).forEach((b) => b.addEventListener("change", () => saveConfig((c) => {
    const x = c.limits.find((y) => y.id === b.dataset.toggle);
    if (x) x.enabled = b.checked;
  })));
  $$("[data-del]", main).forEach((b) => b.addEventListener("click", () => {
    if (window.confirm(t("confirm_delete"))) saveConfig((c) => (c.limits = c.limits.filter((x) => x.id !== b.dataset.del)));
  }));
}
