// Supabase-compatible licensing mock. Implements the same RPC endpoints the
// agent calls on the real project: POST /rest/v1/rpc/{register_device,get_license,redeem_code}.
// Business logic here mirrors the SQL functions in ../supabase/migrations.

const MAX_BODY = 64 * 1024;
const DAY = 86_400_000;

function licenseView(a) {
  return {
    plan: a.plan,
    trial_started: a.trial_started,
    trial_days: a.trial_days,
    premium_until: a.premium_until,
  };
}

/** Recomputes an expired trial to the "expired" plan (premium is sticky). */
function effective(a, now) {
  if (a.plan === "premium") {
    if (!a.premium_until || Date.parse(a.premium_until) > now) return a;
    a.plan = "expired";
    return a;
  }
  const end = Date.parse(a.trial_started) + a.trial_days * DAY;
  a.plan = now < end ? "trial" : "expired";
  return a;
}

export function createApp({ store, apiKey = null, clock = Date.now, log = () => {} } = {}) {
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
    try { return JSON.parse(raw); } catch { throw Object.assign(new Error("bad json"), { status: 400 }); }
  }
  const json = (res, status, obj) => {
    res.writeHead(status, { "Content-Type": "application/json", "Cache-Control": "no-store" });
    res.end(JSON.stringify(obj));
  };

  function registerDevice(b, now) {
    const id = String(b.p_install_id || "");
    if (id.length < 16) throw Object.assign(new Error("install_id"), { status: 400 });
    let a = store.account(id);
    if (!a) {
      a = {
        install_id: id,
        plan: "trial",
        trial_started: new Date(now).toISOString(),
        trial_days: Number(b.p_trial_days) > 0 ? Number(b.p_trial_days) : 4,
        premium_until: null,
        os: String(b.p_os || ""),
        version: String(b.p_version || ""),
        created_at: new Date(now).toISOString(),
      };
    } else {
      a.os = String(b.p_os || a.os);
      a.version = String(b.p_version || a.version);
    }
    effective(a, now);
    store.putAccount(a);
    return licenseView(a);
  }

  function getLicense(b, now) {
    const a = store.account(String(b.p_install_id || ""));
    if (!a) throw Object.assign(new Error("not registered"), { status: 404 });
    effective(a, now);
    store.putAccount(a);
    return licenseView(a);
  }

  function redeemCode(b, now) {
    const a = store.account(String(b.p_install_id || ""));
    if (!a) throw Object.assign(new Error("not registered"), { status: 404 });
    const codeStr = String(b.p_code || "").trim().toUpperCase();
    const code = store.code(codeStr);
    if (!code || code.redeemed_by) throw Object.assign(new Error("invalid or used code"), { status: 400 });
    a.plan = "premium";
    a.premium_until = code.days ? new Date(now + code.days * DAY).toISOString() : null;
    code.redeemed_by = a.install_id;
    code.redeemed_at = new Date(now).toISOString();
    store.putCode(code);
    store.putAccount(a);
    return licenseView(a);
  }

  const RPC = { register_device: registerDevice, get_license: getLicense, redeem_code: redeemCode };

  async function handle(req, res) {
    try {
      const url = new URL(req.url, "http://x");
      if (req.method === "GET" && url.pathname === "/health") return json(res, 200, { ok: true });
      const m = url.pathname.match(/^\/rest\/v1\/rpc\/(\w+)$/);
      if (req.method !== "POST" || !m) return json(res, 404, { code: "not_found" });
      if (apiKey && req.headers["apikey"] !== apiKey) return json(res, 401, { code: "unauthorized" });
      const fn = RPC[m[1]];
      if (!fn) return json(res, 404, { code: "unknown_rpc" });
      const body = await readBody(req);
      const out = fn(body, clock());
      log(`[rpc] ${m[1]} ${body.p_install_id?.slice(0, 8) ?? ""} -> ${out.plan}`);
      return json(res, 200, out);
    } catch (e) {
      return json(res, e.status || 500, { code: e.status ? "bad_request" : "internal", message: e.message });
    }
  }
  return { handle };
}
