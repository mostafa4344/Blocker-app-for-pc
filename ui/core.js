// Shared UI state and helpers.
import { t, setLanguage, getLanguage, num } from "/i18n.js";

export { t, num, getLanguage };

export const S = {
  status: null,
  page: "home",
  offline: false,
  onboarding: false,
  quick: { listId: null, minutes: 60, custom: false, lock: "none" },
  render: () => {},
};

export const $ = (sel, root = document) => root.querySelector(sel);
export const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];

export function esc(v) {
  return String(v ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}

export function newId() {
  const a = new Uint8Array(8);
  crypto.getRandomValues(a);
  return [...a].map((b) => b.toString(16).padStart(2, "0")).join("");
}

export class ApiError extends Error {
  constructor(status, body) {
    super(body?.message || `HTTP ${status}`);
    this.status = status;
    this.code = body?.code;
  }
}

export async function api(method, path, body) {
  const res = await fetch(path, {
    method,
    headers: method === "GET" ? {} : { "Content-Type": "application/json", "X-ContentGuard": "1" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new ApiError(res.status, data);
  return data;
}

export function errMsg(e) {
  if (e?.code) {
    const k = `err_${e.code}`;
    const m = t(k);
    if (m !== k) return m;
  }
  return e?.message || t("error");
}

let toastTimer;
export function toast(msg, isError = false) {
  const el = $("#toast");
  el.textContent = msg;
  el.className = `show${isError ? " error" : ""}`;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (el.className = ""), 3200);
}

const locale = () => (getLanguage() === "bn" ? "bn-BD" : "en-US");

/** Formats a UTC ISO time, or a zone-less local time sent by the agent. */
export function fmtTime(iso) {
  const d = new Date(iso);
  const sameDay = d.toDateString() === new Date().toDateString();
  return d.toLocaleString(locale(), sameDay ? { hour: "numeric", minute: "2-digit" } : { weekday: "short", hour: "numeric", minute: "2-digit" });
}

export function fmtDur(secs) {
  secs = Math.max(0, Math.round(secs));
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${num(h)}${t("hours_short")} ${num(m)}${t("minutes_short")}`;
  if (m > 0) return `${num(m)}${t("minutes_short")}`;
  return `${num(secs)}s`;
}

export const minToHHMM = (m) => `${String(Math.floor(m / 60)).padStart(2, "0")}:${String(m % 60).padStart(2, "0")}`;
export function hhmmToMin(v) {
  const [h, m] = String(v).split(":").map(Number);
  return (h || 0) * 60 + (m || 0);
}
export const lines = (text) => text.split(/[\n,]/).map((s) => s.trim()).filter(Boolean);
export const cfg = () => S.status.config;
export const clone = (o) => JSON.parse(JSON.stringify(o));
export const listName = (id) => cfg().block_lists.find((b) => b.id === id)?.name ?? "?";
export const isLocked = (listId) => S.status.locked_list_ids.includes(listId);

export async function refresh() {
  try {
    S.status = await api("GET", "/api/status");
    S.offline = false;
    setLanguage(S.status.config.settings.language);
  } catch {
    S.offline = true;
  }
}

/** Applies `mutate` to a copy of the config and sends it to the agent. */
export async function saveConfig(mutate, okMsg = t("saved")) {
  const next = clone(cfg());
  mutate(next);
  try {
    await api("PUT", "/api/config", next);
    await refresh();
    S.render();
    if (okMsg) toast(okMsg);
    return true;
  } catch (e) {
    toast(errMsg(e), true);
    return e;
  }
}

/** Runs an API call, refreshes and re-renders; shows errors as a toast. */
export async function act(fn, okMsg) {
  try {
    const r = await fn();
    await refresh();
    S.render();
    if (okMsg) toast(okMsg);
    return r ?? true;
  } catch (e) {
    toast(errMsg(e), true);
    return null;
  }
}

// ---------- modal ----------

export function openModal(html, onMount) {
  closeModal();
  const root = $("#modal-root");
  root.innerHTML = `<div class="modal-backdrop"><div class="modal" role="dialog" aria-modal="true">${html}</div></div>`;
  const backdrop = $(".modal-backdrop", root);
  backdrop.addEventListener("mousedown", (e) => e.target === backdrop && !S.onboarding && closeModal());
  $$("[data-close]", root).forEach((b) => b.addEventListener("click", closeModal));
  onMount?.($(".modal", root));
  const first = $("input, textarea, select, button.primary", root);
  (first || $(".modal", root)).focus();
}

export function closeModal() {
  $("#modal-root").innerHTML = "";
}

export const modalOpen = () => $("#modal-root").children.length > 0;

document.addEventListener("keydown", (e) => e.key === "Escape" && !S.onboarding && closeModal());

/** Shows a random-text challenge; `onAnswer(text)` returns true when accepted. */
export async function challengeModal(purpose, onAnswer) {
  let c;
  try {
    c = await api("POST", "/api/challenge", { purpose });
  } catch (e) {
    return toast(errMsg(e), true);
  }
  openModal(
    `<h2>${t("challenge_title")}</h2>
     <p class="muted">${t("challenge_help")}</p>
     <div class="challenge-text">${esc(c.text)}</div>
     <textarea id="answer" autocomplete="off" spellcheck="false" aria-label="${esc(t("type_challenge"))}"></textarea>
     <div class="actions"><button class="btn" data-close>${t("cancel")}</button><button class="btn primary" id="submit">${t("submit")}</button></div>`,
    (m) => {
      const ta = $("#answer", m);
      ["paste", "drop"].forEach((ev) => ta.addEventListener(ev, (e) => e.preventDefault()));
      $(".challenge-text", m).addEventListener("copy", (e) => e.preventDefault());
      $("#submit", m).addEventListener("click", async () => {
        if (await onAnswer(ta.value)) closeModal();
      });
    },
  );
}

export function updateCountdowns() {
  const now = Date.now();
  $$("[data-countdown]").forEach((el) => {
    el.textContent = fmtDur((new Date(el.dataset.countdown).getTime() - now) / 1000);
  });
}
