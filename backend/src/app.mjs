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
        trial_days: 4, // server decides the trial length (clients cannot choose it)
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
    // Like the SQL: remember version, last check-in and the anonymous health report.
    a.last_seen = new Date(now).toISOString();
    if (b.p_version) a.app_version = String(b.p_version).slice(0, 32);
    if (b.p_health && typeof b.p_health === "object") a.health = b.p_health;
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

  // Admin-managed ready-made lists (Supabase table `preset_lists`, enabled rows, by sort order).
  const getPresets = () =>
    Object.values(store.data.presets ?? {})
      .filter((p) => p.enabled !== false)
      .sort((a, b) => (a.sort_order ?? 100) - (b.sort_order ?? 100))
      .map(({ id, name_en, name_bn, icon, sites, apps, keywords, always_on }) => ({ id, name_en, name_bn, icon, sites, apps, keywords, always_on }));
  const latestRelease = () => {
    const nums = (v) => v.split(".").map(Number);
    const cmp = (a, b) => {
      const x = nums(a.version), y = nums(b.version);
      for (let i = 0; i < Math.max(x.length, y.length); i++) if ((x[i] ?? 0) !== (y[i] ?? 0)) return (y[i] ?? 0) - (x[i] ?? 0);
      return 0;
    };
    const r = Object.values(store.data.releases ?? {}).filter((r) => r.enabled !== false && r.rollout > 0).sort(cmp)[0];
    return r ? { version: r.version, url: r.url, sha256: r.sha256, size: r.size, signature: r.signature, rollout: r.rollout, notes: r.notes ?? "", notes_bn: r.notes_bn ?? "" } : null;
  };
  const presetsVersion = () => {
    const txt = JSON.stringify(store.data.presets ?? {});
    let h = 0;
    for (let i = 0; i < txt.length; i++) h = (h * 31 + txt.charCodeAt(i)) | 0;
    return `${(h >>> 0).toString(16)}-${Object.keys(store.data.presets ?? {}).length}`;
  };
  // Like the SQL submit_purchase_request(): a pending request the admin approves.
  const submitPurchase = (b) => {
    if (!store.account(String(b.p_install_id || ""))) throw Object.assign(new Error("not registered"), { status: 404 });
    const txn = String(b.p_txn_id || "").trim().toUpperCase();
    if (!/^[A-Z0-9]{4,40}$/.test(txn)) throw Object.assign(new Error("bad txn"), { status: 400 });
    const list = (store.data.purchase_requests ??= []);
    if (list.some((r) => r.txn_id === txn && r.status === "approved")) throw Object.assign(new Error("txn already used"), { status: 400 });
    list.push({ install_id: b.p_install_id, plan: b.p_plan, method: b.p_method, txn_id: txn, phone: b.p_contact || null, status: "pending", created_at: new Date().toISOString() });
    store.save();
    return { ok: true };
  };
  const RPC = { register_device: registerDevice, get_license: getLicense, redeem_code: redeemCode, get_presets: getPresets, submit_purchase_request: submitPurchase };

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
      // Like the SQL license_json(): every license answer carries the preset-list
      // version, the newest release and the health-report flag.
      if (out && !Array.isArray(out) && "plan" in out) {
        out.presets_version = presetsVersion();
        out.latest_release = latestRelease();
        out.accepts_health = true;
        out.purchase_url = store.data.config?.purchase_url || null;
        const c = store.data.config ?? {};
        out.pay = { bkash: c.bkash_number ?? "01568689505", nagad: c.nagad_number ?? "01568689505", rocket: c.rocket_number ?? null,
                    monthly: c.price_monthly ?? "৳199", yearly: c.price_yearly ?? "৳999", lifetime: c.price_lifetime ?? "৳1999" };
      }
      log(`[rpc] ${m[1]} ${body.p_install_id?.slice(0, 8) ?? ""} -> ${out?.plan ?? (Array.isArray(out) ? `${out.length} items` : "")}`);
      return json(res, 200, out);
    } catch (e) {
      return json(res, e.status || 500, { code: e.status ? "bad_request" : "internal", message: e.message });
    }
  }
  return { handle };
}
