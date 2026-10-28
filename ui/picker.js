// "Choose a website or app": one picker used by Focus, Schedules and Limits.
// People pick what they actually use (from this computer's usage), an app
// installed here (by its friendly name; the real program file is filled in),
// or type any website. Behind the scenes each pick gets its own small block
// list, so schedules, limits and locks work on it exactly like on any list.
import { S, $, $$, t, esc, api, cfg, fmtDur, saveConfig } from "/core.js";

/** Suggestions from the agent, cached for a minute. */
async function suggestions() {
  if (S.pickData && Date.now() - S.pickData.at < 60_000) return S.pickData;
  try {
    S.pickData = { at: Date.now(), ...(await api("GET", "/api/apps")) };
  } catch {
    S.pickData = { at: Date.now(), top: [], installed: [], running: [] };
  }
  return S.pickData;
}

/** "https://www.YouTube.com/shorts/" -> "youtube.com/shorts"; null if not a site. */
export function cleanSite(text) {
  const s = String(text || "").trim().toLowerCase()
    .replace(/^[a-z]+:\/\//, "").replace(/^www\./, "").replace(/[?#].*$/, "").replace(/\/+$/, "");
  return /^[a-z0-9-]+(\.[a-z0-9-]+)+(\/[^\s]*)?$/.test(s) ? s : null;
}

// A website is named by its address: clear in every language and country.
const siteName = (site) => site;

/** The block list for one site or app: an existing one-item list, or a new one. */
export async function ensureTargetList(target) {
  const lists = cfg().block_lists;
  const value = target.value.toLowerCase();
  const found = lists.find((l) => !l.always_on && !l.allowlist_mode
    && (target.kind === "site"
      ? l.sites.length === 1 && l.sites[0].toLowerCase() === value && l.apps.length === 0
      : l.apps.length === 1 && l.apps[0].toLowerCase() === value && l.sites.length === 0));
  if (found) return found.id;
  const id = `t-${value.replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 40)}-${Math.random().toString(36).slice(2, 6)}`;
  const ok = await saveConfig((c) => c.block_lists.push({
    id, name: target.name.slice(0, 60), preset: "", always_on: false, allowlist_mode: false,
    sites: target.kind === "site" ? [value] : [], apps: target.kind === "app" ? [value] : [], keywords: [],
  }), null);
  return ok === true ? id : null;
}

export function pickerHTML() {
  return `<div class="picker stack">
      <input type="search" class="picker-q" autocomplete="off" spellcheck="false"
        placeholder="${esc(t("picker_search"))}" aria-label="${esc(t("picker_search"))}" value="${esc(S.pickQuery || "")}">
      <div class="picker-results" aria-live="polite"></div>
    </div>`;
}

function option(target, extra = "") {
  const icon = target.kind === "site" ? "🌐" : "🧩";
  return `<button type="button" class="pick-opt" data-kind="${target.kind}" data-value="${esc(target.value)}" data-name="${esc(target.name)}">
      <span aria-hidden="true">${icon}</span><span class="pick-name">${esc(target.name)}</span>${extra ? `<small class="muted">${esc(extra)}</small>` : ""}
    </button>`;
}

function results(data, q) {
  const query = q.trim().toLowerCase();
  const out = [];
  const seen = new Set();
  const add = (target, extra) => {
    const key = `${target.kind}:${target.value.toLowerCase()}`;
    if (seen.has(key)) return;
    seen.add(key);
    out.push(option(target, extra));
  };
  const top = data.top.map((x) => x.kind === "site"
    ? [{ kind: "site", value: x.name, name: x.name }, fmtDur(x.seconds)]
    : [{ kind: "app", value: x.name.endsWith(".exe") ? x.name : `${x.name}.exe`, name: x.name.replace(/\.exe$/i, "") }, fmtDur(x.seconds)]);
  const apps = [...data.installed, ...data.running].map((a) => [{ kind: "app", value: a.exe, name: a.name }, a.exe]);
  if (!query) {
    const section = (title, items) => items.length ? `<div class="pick-sec"><strong>${title}</strong><div class="pick-list">${items.join("")}</div></div>` : "";
    const topHtml = top.map(([tg, ex]) => option(tg, ex));
    const appHtml = apps.slice(0, 40).map(([tg, ex]) => option(tg, ex));
    return section(t("picker_top"), topHtml) + section(t("picker_installed"), appHtml)
      || `<p class="muted">${t("picker_empty")}</p>`;
  }
  const site = cleanSite(query);
  if (site) add({ kind: "site", value: site, name: site }, t("picker_any_site"));
  for (const [tg, ex] of [...top, ...apps]) {
    if (tg.name.toLowerCase().includes(query) || tg.value.toLowerCase().includes(query)) add(tg, ex);
  }
  if (/^[\w .+-]+\.exe$/i.test(query)) add({ kind: "app", value: query, name: query.replace(/\.exe$/i, "") }, query);
  if (!out.length) {
    return `<p class="muted">${t("picker_none")}</p>`;
  }
  return `<div class="pick-list">${out.slice(0, 30).join("")}</div>`;
}

/** Wires a picker rendered with `pickerHTML()`; `onPick(target)` gets {kind, value, name}. */
export async function mountPicker(root, onPick) {
  const input = $(".picker-q", root);
  const box = $(".picker-results", root);
  if (!input || !box) return;
  box.innerHTML = `<p class="muted">…</p>`;
  const data = await suggestions();
  const draw = () => {
    S.pickQuery = input.value;
    box.innerHTML = results(data, input.value);
    $$(".pick-opt", box).forEach((b) => b.addEventListener("click", () => {
      const v = b.dataset.value;
      const name = b.dataset.kind === "site" ? siteName(v) : b.dataset.name;
      S.pickQuery = "";
      onPick({ kind: b.dataset.kind, value: v, name });
    }));
  };
  input.addEventListener("input", draw);
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      $(".pick-opt", box)?.click();
    }
  });
  draw();
}
