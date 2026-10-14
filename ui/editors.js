// Block lists, schedules and daily limits.
import { S, $, $$, t, num, esc, api, act, cfg, fmtDur, openModal, closeModal, saveConfig, lines, newId, isLocked, listName, minToHHMM, hhmmToMin } from "/core.js";

const lockBadge = (on) => (on ? `<span class="badge lock">🔒 ${t("locked")}</span>` : "");

// ---------- block lists ----------

function editList(list) {
  const isNew = !list;
  const l = list ?? { id: newId(), name: "", sites: [], apps: [], keywords: [], allowlist_mode: false };
  const locked = !isNew && isLocked(l.id);
  openModal(
    `<h2>${isNew ? t("add_list") : esc(l.name)}</h2>
     ${locked ? `<div class="banner warn">${t("list_locked_note")}</div>
` : ""}
     <label>${t("name")}<input type="text" id="l-name" value="${esc(l.name)}" maxlength="60"></label>
     <label>${t("sites")}<span class="hint">${t("sites_hint")}</span><textarea id="l-sites">${esc(l.sites.join("\n"))}</textarea></label>
     <label>${t("apps")}<span class="hint">${t("apps_hint")}</span><textarea id="l-apps">${esc(l.apps.join("\n"))}</textarea></label>
     <label>${t("keywords")}<textarea id="l-keywords">${esc(l.keywords.join("\n"))}</textarea></label>
     <label class="check"><input type="checkbox" id="l-allow" ${l.allowlist_mode ? "checked" : ""} ${locked ? "disabled" : ""}>${t("allowlist_mode")}</label>
     <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="l-save">${t("save")}</button></div>`,
    (m) => {
      $("#l-save", m).addEventListener("click", async () => {
        const updated = {
          ...l,
          name: $("#l-name", m).value.trim() || "List",
          sites: lines($("#l-sites", m).value),
          apps: lines($("#l-apps", m).value),
          keywords: lines($("#l-keywords", m).value),
          allowlist_mode: $("#l-allow", m).checked,
        };
        const ok = await saveConfig((c) => {
          const i = c.block_lists.findIndex((b) => b.id === l.id);
          if (i >= 0) c.block_lists[i] = updated;
          else c.block_lists.push(updated);
        });
        if (ok === true) closeModal();
      });
    },
  );
}

function presetPicker() {
  const presets = S.status.presets;
  const bn = cfg().settings.language === "bn";
  openModal(
    `<h2>${t("add_preset")}</h2>
     <div class="preset-grid">${presets.map((p) => `<button class="preset" data-preset="${esc(p.id)}"><span class="icon">${p.icon}</span><strong>${esc(bn ? p.name_bn : p.name_en)}</strong><small>${num(p.sites.length)} · ${num(p.apps.length)}</small></button>`).join("")}</div>
     <div class="actions"><button class="btn" data-close>${t("close")}</button></div>`,
    (m) => $$("[data-preset]", m).forEach((b) => b.addEventListener("click", async () => {
      if (await act(() => api("POST", `/api/presets/${b.dataset.preset}`), t("saved"))) closeModal();
    })),
  );
}

export function renderLists(main) {
  const lists = cfg().block_lists;
  main.innerHTML = `
    <div class="page-head"><h1>${t("lists_title")}</h1>
      <div class="row"><button class="btn" id="add-preset">${t("add_preset")}</button><button class="btn primary" id="add-list">+ ${t("add_list")}</button></div></div>
    <div class="list">
      ${lists.length === 0 ? `<p class="muted">${t("no_data")}</p>` : lists.map((l) => `
        <div class="item">
          <div><div class="row"><strong>${esc(l.name)}</strong>${lockBadge(isLocked(l.id))}${l.allowlist_mode ? `<span class="badge">allowlist</span>` : ""}</div>
            <div class="meta">${t("items", { sites: num(l.sites.length), apps: num(l.apps.length) })}</div></div>
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

function listOptions(selected) {
  return cfg().block_lists.map((l) => `<option value="${esc(l.id)}" ${l.id === selected ? "selected" : ""}>${esc(l.name)}</option>`).join("");
}

function daysSummary(days) {
  if (days.length === 7) return t("weekdays_all");
  const names = t("day_names");
  return [...days].sort().map((d) => names[d]).join(", ");
}

function editSchedule(sched) {
  const isNew = !sched;
  const s = sched ?? { id: newId(), block_list_id: cfg().block_lists[0]?.id, days: [0, 1, 2, 3, 4], start_minute: 540, end_minute: 1020, enabled: true, locked: false };
  const names = t("day_names");
  openModal(
    `<h2>${isNew ? t("add_schedule") : t("schedules_title")}</h2>
     ${s.locked && !isNew ? `<div class="banner warn">${t("lock_schedule")}</div>` : ""}
     <label>${t("choose_list")}<select id="s-list" ${!isNew && s.locked ? "disabled" : ""}>${listOptions(s.block_list_id)}</select></label>
     <div class="stack"><span class="hint">${t("days")}</span><div class="chips">${names.map((n, i) => `<button class="chip" data-day="${i}" aria-pressed="${s.days.includes(i)}">${n}</button>`).join("")}</div></div>
     <div class="grid two"><label>${t("start_time")}<input type="time" id="s-start" value="${minToHHMM(s.start_minute)}"></label>
       <label>${t("end_time")}<input type="time" id="s-end" value="${minToHHMM(s.end_minute)}"></label></div>
     <label class="check"><input type="checkbox" id="s-locked" ${s.locked ? "checked" : ""}>${t("lock_schedule")}</label>
     <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="s-save">${t("save")}</button></div>`,
    (m) => {
      const days = new Set(s.days);
      $$("[data-day]", m).forEach((b) => b.addEventListener("click", () => {
        const d = Number(b.dataset.day);
        days.has(d) ? days.delete(d) : days.add(d);
        b.setAttribute("aria-pressed", days.has(d));
      }));
      $("#s-save", m).addEventListener("click", async () => {
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
  const noLists = cfg().block_lists.length === 0;
  main.innerHTML = `
    <div class="page-head"><h1>${t("schedules_title")}</h1><button class="btn primary" id="add" ${noLists ? "disabled" : ""}>+ ${t("add_schedule")}</button></div>
    <div class="list">${items.length === 0 ? `<p class="muted">${t("no_data")}</p>` : items.map((s) => `
      <div class="item">
        <div><div class="row"><strong>${esc(listName(s.block_list_id))}</strong>${lockBadge(s.locked)}</div>
          <div class="meta">${daysSummary(s.days)} · ${num(minToHHMM(s.start_minute))}–${num(minToHHMM(s.end_minute))}</div></div>
        <div class="row">
          <label class="check"><input type="checkbox" data-toggle="${esc(s.id)}" ${s.enabled ? "checked" : ""} ${s.locked ? "disabled" : ""}>${t("enabled")}</label>
          <button class="btn small" data-edit="${esc(s.id)}">${t("edit")}</button>
          ${s.locked ? "" : `<button class="btn small danger" data-del="${esc(s.id)}">${t("delete")}</button>`}
        </div>
      </div>`).join("")}</div>`;
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

function editLimit(limit) {
  const isNew = !limit;
  const l = limit ?? { id: newId(), block_list_id: cfg().block_lists[0]?.id, daily_minutes: 30, max_opens: null, enabled: true, locked: false };
  openModal(
    `<h2>${isNew ? t("add_limit") : t("limits_title")}</h2>
     <label>${t("choose_list")}<select id="m-list" ${!isNew && l.locked ? "disabled" : ""}>${listOptions(l.block_list_id)}</select></label>
     <label>${t("daily_minutes")}<input type="number" id="m-min" min="0" max="1440" value="${l.daily_minutes}"></label>
     <label>${t("max_opens")}<input type="number" id="m-opens" min="1" max="500" value="${l.max_opens ?? ""}"></label>
     <label class="check"><input type="checkbox" id="m-locked" ${l.locked ? "checked" : ""}>${t("lock_limit")}</label>
     <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="m-save">${t("save")}</button></div>`,
    (m) => $("#m-save", m).addEventListener("click", async () => {
      const opens = $("#m-opens", m).value.trim();
      const updated = {
        ...l,
        block_list_id: $("#m-list", m).value,
        daily_minutes: Math.max(0, Math.round(Number($("#m-min", m).value) || 0)),
        max_opens: opens ? Math.max(1, Math.round(Number(opens))) : null,
        locked: $("#m-locked", m).checked,
      };
      const ok = await saveConfig((c) => {
        const i = c.limits.findIndex((x) => x.id === l.id);
        if (i >= 0) c.limits[i] = updated;
        else c.limits.push(updated);
      });
      if (ok === true) closeModal();
    }),
  );
}

export function renderLimits(main) {
  const items = cfg().limits;
  const usage = S.status.usage_today;
  const noLists = cfg().block_lists.length === 0;
  main.innerHTML = `
    <div class="page-head"><h1>${t("limits_title")}</h1><button class="btn primary" id="add" ${noLists ? "disabled" : ""}>+ ${t("add_limit")}</button></div>
    <div class="list">${items.length === 0 ? `<p class="muted">${t("no_data")}</p>` : items.map((l) => {
      const u = usage.find((x) => x.limit_id === l.id) ?? { seconds: 0, opens: 0 };
      const pct = l.daily_minutes ? Math.min(100, Math.round((u.seconds / (l.daily_minutes * 60)) * 100)) : 100;
      return `
      <div class="item">
        <div class="stack">
          <div class="row"><strong>${esc(listName(l.block_list_id))}</strong>${lockBadge(l.locked)}</div>
          <div class="meta">${t("used_today")}: ${fmtDur(u.seconds)} / ${fmtDur(l.daily_minutes * 60)}${l.max_opens ? ` · ${num(u.opens)}/${num(l.max_opens)} ${t("opens")}` : ""}</div>
          <div class="progress ${pct >= 100 ? "full" : ""}"><div data-w="${pct}"></div></div>
        </div>
        <div class="row">
          <label class="check"><input type="checkbox" data-toggle="${esc(l.id)}" ${l.enabled ? "checked" : ""} ${l.locked ? "disabled" : ""}>${t("enabled")}</label>
          <button class="btn small" data-edit="${esc(l.id)}">${t("edit")}</button>
          ${l.locked ? "" : `<button class="btn small danger" data-del="${esc(l.id)}">${t("delete")}</button>`}
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
