// HTTP handlers for the ContentGuard partner/sync backend.
import crypto from "node:crypto";

const SILENT_AFTER_MS = 30 * 60 * 1000;
const MAX_BODY = 64 * 1024;
const LEVELS = ["off", "mindful", "committed", "partner"];
const KINDS = ["uninstall", "change_protection", "end_session"];

const sha256 = (s) => crypto.createHash("sha256").update(s).digest("hex");
const token = () => crypto.randomBytes(32).toString("base64url");
const nowIso = (clock) => new Date(clock()).toISOString();

function safeEqual(a, b) {
  const x = Buffer.from(a);
  const y = Buffer.from(b);
  return x.length === y.length && crypto.timingSafeEqual(x, y);
}

function escapeHtml(s) {
  return String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}

const isEmail = (s) => typeof s === "string" && s.length <= 254 && /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s);
const str = (v, max) => (typeof v === "string" ? v.slice(0, max) : "");

const TEXT = {
  bn: {
    kind: { uninstall: "ContentGuard আনইন্সটল করা", change_protection: "সুরক্ষা কমানো", end_session: "লক করা ফোকাস সেশন থামানো" },
    reqSubject: (dev) => `${dev}: আপনার অনুমতি চাওয়া হয়েছে`,
    reqBody: (who, dev, kind, reason, link, auto) =>
      `${who} তাঁর কম্পিউটার "${dev}"-এ ${kind}-এর অনুমতি চেয়েছেন।\n${reason ? `কারণ: ${reason}\n` : ""}\nঅনুমতি দিতে বা না দিতে এখানে যান: ${link}\n\nআপনি উত্তর না দিলে ${auto}-এ নিজে থেকে অনুমোদিত হবে।`,
    silentSubject: (dev) => `${dev}: ContentGuard রিপোর্ট পাঠানো বন্ধ করেছে`,
    silentBody: (dev, last) => `"${dev}" কম্পিউটারের ContentGuard শেষবার ${last}-এ যোগাযোগ করেছিল। কম্পিউটার বন্ধ বা অফলাইন থাকতে পারে, অথবা ContentGuard বন্ধ করা হয়ে থাকতে পারে। একবার খোঁজ নিয়ে দেখতে পারেন।`,
    uninstSubject: (dev) => `${dev}: ContentGuard আনইন্সটল করা হয়েছে`,
    uninstBody: (dev) => `অনুমোদিত অনুরোধ অনুযায়ী "${dev}" থেকে ContentGuard সরানো হয়েছে।`,
    page: { title: "অনুমতির অনুরোধ", approve: "অনুমতি দিন", deny: "অনুমতি দেবেন না", done: "ধন্যবাদ! আপনার সিদ্ধান্ত পাঠানো হয়েছে।", gone: "এই লিংকটি আর কাজ করছে না।", reason: "কারণ" },
  },
  en: {
    kind: { uninstall: "uninstall ContentGuard", change_protection: "lower their protection", end_session: "end a locked focus session" },
    reqSubject: (dev) => `${dev}: your approval is requested`,
    reqBody: (who, dev, kind, reason, link, auto) =>
      `${who} asked for your approval to ${kind} on "${dev}".\n${reason ? `Reason: ${reason}\n` : ""}\nApprove or deny here: ${link}\n\nIf you don't answer, it will be approved automatically at ${auto}.`,
    silentSubject: (dev) => `${dev}: ContentGuard stopped reporting`,
    silentBody: (dev, last) => `ContentGuard on "${dev}" last checked in at ${last}. The computer may be off or offline, or ContentGuard may have been stopped. You may want to check in.`,
    uninstSubject: (dev) => `${dev}: ContentGuard was uninstalled`,
    uninstBody: (dev) => `ContentGuard was removed from "${dev}" after an approved request.`,
    page: { title: "Approval request", approve: "Approve", deny: "Deny", done: "Thank you! Your decision was sent.", gone: "This link is no longer valid.", reason: "Reason" },
  },
};
const T = (dev) => TEXT[dev?.language === "en" ? "en" : "bn"];

export function createApp({ store, publicUrl = "http://localhost:8787", clock = Date.now, log = console.log } = {}) {
  const email = (to, subject, body) => {
    if (!isEmail(to)) return;
    store.insertOutbox({ id: crypto.randomUUID(), to, subject, body, created_at: nowIso(clock), sent_at: null });
    log(`[outbox] to=${to} subject=${subject}`);
  };

  function auth(req, deviceId) {
    const dev = store.getDevice(deviceId);
    const h = req.headers.authorization || "";
    if (!dev || !h.startsWith("Bearer ")) return null;
    return safeEqual(sha256(h.slice(7)), dev.secret_hash) ? dev : null;
  }

  async function readBody(req) {
    let size = 0;
    const chunks = [];
    for await (const c of req) {
      size += c.length;
      if (size > MAX_BODY) throw Object.assign(new Error("too large"), { status: 413 });
      chunks.push(c);
    }
    const raw = Buffer.concat(chunks).toString("utf8");
    if (!raw) return {};
    if ((req.headers["content-type"] || "").includes("application/x-www-form-urlencoded")) return Object.fromEntries(new URLSearchParams(raw));
    try {
      return JSON.parse(raw);
    } catch {
      throw Object.assign(new Error("invalid json"), { status: 400 });
    }
  }

  const json = (res, status, obj) => {
    res.writeHead(status, { "Content-Type": "application/json; charset=utf-8", "Cache-Control": "no-store" });
    res.end(JSON.stringify(obj));
  };
  const html = (res, status, body, lang = "bn") => {
    res.writeHead(status, {
      "Content-Type": "text/html; charset=utf-8",
      "Content-Security-Policy": "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'",
      "X-Frame-Options": "DENY",
      "Referrer-Policy": "no-referrer",
    });
    res.end(`<!doctype html><html lang="${lang}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ContentGuard</title>
<style>body{font-family:system-ui,"Noto Sans Bengali",sans-serif;background:#f6f7fb;display:grid;place-items:center;min-height:100vh;margin:0;padding:16px}main{background:#fff;padding:32px;border-radius:16px;max-width:480px;box-shadow:0 8px 30px #0001}button{font:inherit;padding:10px 18px;border-radius:10px;border:1px solid #ccd;margin:6px;cursor:pointer}.ok{background:#4f46e5;color:#fff;border-color:#4f46e5}</style></head><body><main>${body}</main></body></html>`);
  };

  /** Auto-approve requests whose deadline passed (mirrors the agent's own timeout). */
  function effectiveStatus(r) {
    if (r.status === "pending" && r.auto_approve_at && Date.parse(r.auto_approve_at) <= clock()) return "approved";
    return r.status;
  }

  async function handle(req, res) {
    const url = new URL(req.url, "http://x");
    const seg = url.pathname.split("/").filter(Boolean);
    const m = req.method;
    try {
      if (m === "GET" && url.pathname === "/health") return json(res, 200, { ok: true });

      // Register a device.
      if (m === "POST" && seg[0] === "v1" && seg[1] === "devices" && seg.length === 2) {
        const b = await readBody(req);
        const secret = token();
        const dev = store.insertDevice({
          id: crypto.randomUUID(),
          secret_hash: sha256(secret),
          name: str(b.name, 80) || "PC",
          os: str(b.os, 20),
          version: str(b.version, 20),
          protection_level: "off",
          tamper_alerts: false,
          partner_name: null,
          partner_email: null,
          language: "bn",
          last_heartbeat_at: null,
          alert_sent_at: null,
          uninstalled_at: null,
          created_at: nowIso(clock),
        });
        return json(res, 201, { device_id: dev.id, device_secret: secret });
      }

      if (seg[0] === "v1" && seg[1] === "devices" && seg[2]) {
        const dev = auth(req, seg[2]);
        if (!dev) return json(res, 401, { code: "unauthorized" });
        const rest = seg.slice(3);

        if (m === "PUT" && rest.length === 0) {
          const b = await readBody(req);
          const patch = {
            protection_level: LEVELS.includes(b.protection_level) ? b.protection_level : dev.protection_level,
            tamper_alerts: Boolean(b.tamper_alerts),
            partner_name: str(b.partner_name, 80) || null,
            partner_email: isEmail(b.partner_email) ? b.partner_email : null,
            language: b.language === "en" ? "en" : "bn",
          };
          return json(res, 200, { ok: true, device: { ...store.updateDevice(dev.id, patch), secret_hash: undefined } });
        }
        if (m === "POST" && rest[0] === "heartbeat") {
          const b = await readBody(req);
          store.updateDevice(dev.id, { last_heartbeat_at: nowIso(clock), version: str(b.version, 20) || dev.version, uninstalled_at: null });
          return json(res, 200, { ok: true });
        }
        if (m === "POST" && rest[0] === "uninstalled") {
          store.updateDevice(dev.id, { uninstalled_at: nowIso(clock) });
          email(dev.partner_email, T(dev).uninstSubject(dev.name), T(dev).uninstBody(dev.name));
          return json(res, 200, { ok: true });
        }
        if (m === "POST" && rest[0] === "requests" && rest.length === 1) {
          const b = await readBody(req);
          const id = str(b.request_id, 64);
          const kind = b.kind?.type;
          if (!id || !KINDS.includes(kind)) return json(res, 400, { code: "bad_request" });
          const existing = store.getRequest(dev.id, id);
          if (existing) return json(res, 200, { ok: true, status: effectiveStatus(existing) });
          const tok = token();
          const autoAt = b.auto_approve_at && !Number.isNaN(Date.parse(b.auto_approve_at)) ? new Date(b.auto_approve_at).toISOString() : null;
          store.insertRequest({ id, device_id: dev.id, kind, reason: str(b.reason, 500), auto_approve_at: autoAt, status: "pending", token_hash: sha256(tok), decided_at: null, created_at: nowIso(clock) });
          const t = T(dev);
          const link = `${publicUrl.replace(/\/$/, "")}/approve/${tok}`;
          const auto = autoAt ? new Date(autoAt).toUTCString() : "-";
          email(dev.partner_email, t.reqSubject(dev.name), t.reqBody(dev.name, dev.name, t.kind[kind], str(b.reason, 500), link, auto));
          return json(res, 201, { ok: true, status: "pending" });
        }
        if (m === "GET" && rest[0] === "requests" && rest[1]) {
          const r = store.getRequest(dev.id, rest[1]);
          if (!r) return json(res, 404, { code: "not_found" });
          return json(res, 200, { id: r.id, status: effectiveStatus(r), decided_at: r.decided_at });
        }
        return json(res, 404, { code: "not_found" });
      }

      // Partner approval page (link from the email).
      if (seg[0] === "approve" && seg[1] && seg.length === 2) {
        const r = store.getRequestByTokenHash(sha256(seg[1]));
        const dev = r ? store.getDevice(r.device_id) : null;
        const t = T(dev);
        if (!r || !dev || effectiveStatus(r) !== "pending") return html(res, 410, `<h1>${t.page.gone}</h1>`);
        if (m === "POST") {
          const b = await readBody(req);
          const decision = b.decision === "approve" ? "approved" : b.decision === "deny" ? "denied" : null;
          if (!decision) return html(res, 400, `<h1>${t.page.gone}</h1>`);
          store.updateRequest(r.device_id, r.id, { status: decision, decided_at: nowIso(clock) });
          return html(res, 200, `<h1>${t.page.done}</h1>`);
        }
        if (m === "GET") {
          return html(res, 200, `<h1>${t.page.title}</h1><p><strong>${escapeHtml(dev.name)}</strong>: ${escapeHtml(t.kind[r.kind])}</p>
            ${r.reason ? `<p>${t.page.reason}: “${escapeHtml(r.reason)}”</p>` : ""}
            <form method="post"><button class="ok" name="decision" value="approve">${t.page.approve}</button><button name="decision" value="deny">${t.page.deny}</button></form>`, dev.language);
        }
      }
      return json(res, 404, { code: "not_found" });
    } catch (e) {
      return json(res, e.status || 500, { code: e.status ? "bad_request" : "internal", message: e.status ? e.message : undefined });
    }
  }

  /** Emails the partner once when a protected device stops sending heartbeats. */
  function checkSilentDevices() {
    const now = clock();
    for (const d of store.listDevices()) {
      if (!d.tamper_alerts || d.uninstalled_at || !d.last_heartbeat_at) continue;
      const last = Date.parse(d.last_heartbeat_at);
      if (now - last < SILENT_AFTER_MS) continue;
      if (d.alert_sent_at && Date.parse(d.alert_sent_at) >= last) continue;
      email(d.partner_email, T(d).silentSubject(d.name), T(d).silentBody(d.name, new Date(last).toUTCString()));
      store.updateDevice(d.id, { alert_sent_at: nowIso(clock) });
    }
  }

  return { handle, checkSilentDevices };
}
