// ============================================================================
// Dashboard single-file version — shared helpers inlined. Paste this whole file
// as the function's index.ts in the Supabase dashboard (Edge Functions editor).
// ============================================================================
import { createClient } from "https://esm.sh/@supabase/supabase-js@2.45.4";
import { create, verify, getNumericDate } from "https://deno.land/x/djwt@v3.0.2/mod.ts";

const corsHeaders = {
  "Access-Control-Allow-Origin": "*", // TODO: set to your web origin in prod
  "Access-Control-Allow-Headers": "authorization, x-client-info, apikey, content-type, x-device-jwt",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
};
function handleOptions(): Response { return new Response("ok", { headers: corsHeaders }); }
function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status, headers: { ...corsHeaders, "Content-Type": "application/json" },
  });
}
function serviceClient() {
  return createClient(Deno.env.get("SUPABASE_URL")!, Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!, {
    auth: { persistSession: false, autoRefreshToken: false },
  });
}
const _enc = new TextEncoder();
async function _jwtKey(): Promise<CryptoKey> {
  const secret = Deno.env.get("DEVICE_JWT_SECRET");
  if (!secret) throw new Error("DEVICE_JWT_SECRET not set");
  return await crypto.subtle.importKey("raw", _enc.encode(secret),
    { name: "HMAC", hash: "SHA-256" }, false, ["sign", "verify"]);
}
async function mintDeviceJwt(deviceId: string): Promise<string> {
  const ttlDays = Number(Deno.env.get("DEVICE_JWT_TTL_DAYS") ?? "30");
  return await create({ alg: "HS256", typ: "JWT" },
    { sub: deviceId, device_id: deviceId, scope: "device", exp: getNumericDate(ttlDays * 86400) },
    await _jwtKey());
}
async function verifyDeviceJwt(token: string): Promise<string | null> {
  try {
    const p = await verify(token, await _jwtKey());
    if (p.scope !== "device" || typeof p.device_id !== "string") return null;
    return p.device_id;
  } catch { return null; }
}
function deviceTokenFromRequest(req: Request): string | null { return req.headers.get("x-device-jwt"); }
// ============================================================================
// device-sync
// ----------------------------------------------------------------------------
// The device's main polling endpoint. Authenticated by the per-device JWT.
//   * Pulls the active policy for the device's child (if newer than device has).
//   * Pulls pending commands (lock_now, unlink, ...).
//   * Accepts a batch of usage_events to store.
//   * Updates device status (last_seen, battery, foreground app).
//
// The device_id ALWAYS comes from the verified JWT, never from the body.

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return handleOptions();
  if (req.method !== "POST") return json({ error: "method_not_allowed" }, 405);

  const token = deviceTokenFromRequest(req);
  if (!token) return json({ error: "missing_device_jwt" }, 401);
  const deviceId = await verifyDeviceJwt(token);
  if (!deviceId) return json({ error: "invalid_device_jwt" }, 401);

  let body: {
    known_policy_version?: number;
    status?: { battery_pct?: number; foreground_app?: string; push_token?: string };
    usage?: Array<{ package_name: string; foreground_s: number; occurred_at: string }>;
    locations?: Array<{ lat: number; lon: number; accuracy_m?: number; occurred_at: string }>;
    ack_command_ids?: string[];
  } = {};
  try {
    if (req.headers.get("content-length") !== "0") body = await req.json();
  } catch {
    return json({ error: "bad_json" }, 400);
  }

  const db = serviceClient();

  // Reject revoked devices.
  const { data: device, error: dErr } = await db
    .from("devices")
    .select("id, child_id, revoked_at")
    .eq("id", deviceId)
    .maybeSingle();
  if (dErr) return json({ error: "server_error" }, 500);
  if (!device) return json({ error: "unknown_device" }, 404);
  if (device.revoked_at) return json({ error: "revoked", action: "self_disable" }, 403);

  // Ack commands the device has applied (before fetching pending ones, so acked
  // commands are not returned again). Scoped to this device.
  if (Array.isArray(body.ack_command_ids) && body.ack_command_ids.length > 0) {
    await db.from("device_commands")
      .update({ acked_at: new Date().toISOString() })
      .eq("device_id", deviceId)
      .in("id", body.ack_command_ids.slice(0, 100));
  }

  // Update live status. Only overwrite push_token when one was sent.
  const statusUpdate: Record<string, unknown> = {
    last_seen_at: new Date().toISOString(),
    battery_pct: body.status?.battery_pct ?? null,
    foreground_app: body.status?.foreground_app ?? null,
  };
  if (body.status?.push_token) statusUpdate.push_token = body.status.push_token;
  await db.from("devices").update(statusUpdate).eq("id", deviceId);

  // Store opt-in location points (the parent enables this; policy gates sending).
  if (Array.isArray(body.locations) && body.locations.length > 0) {
    const locRows = body.locations.slice(0, 50)
      .filter((l) => Number.isFinite(l.lat) && Number.isFinite(l.lon))
      .map((l) => ({
        device_id: deviceId,
        lat: l.lat,
        lon: l.lon,
        accuracy_m: Number.isFinite(l.accuracy_m as number) ? l.accuracy_m : null,
        occurred_at: l.occurred_at ?? new Date().toISOString(),
      }));
    if (locRows.length > 0) await db.from("locations").insert(locRows);
  }

  // Store usage batch (bounded; validate before trusting). The device sends
  // today's cumulative seconds per package; UPSERT onto the (device,package,day)
  // row so repeated syncs overwrite instead of accumulating.
  if (Array.isArray(body.usage) && body.usage.length > 0) {
    const today = new Date().toISOString().slice(0, 10); // UTC day
    const nowIso = new Date().toISOString();
    const rows = body.usage.slice(0, 500).map((u) => ({
      device_id: deviceId,
      package_name: String(u.package_name).slice(0, 255),
      foreground_s: Math.max(0, Math.min(86400, Number(u.foreground_s) || 0)),
      day: today,
      occurred_at: nowIso,
    }));
    await db.from("usage_events").upsert(rows, { onConflict: "device_id,package_name,day" });
  }

  // Active policy (only return the doc if the device's version is stale).
  const { data: policy } = await db
    .from("policies")
    .select("version, daily_limit_min, bedtime_start, bedtime_end, timezone, location_enabled, web_filter_level, rules")
    .eq("child_id", device.child_id)
    .eq("is_active", true)
    .maybeSingle();

  const policyChanged = policy && policy.version !== (body.known_policy_version ?? -1);

  // Pending commands.
  const { data: commands } = await db
    .from("device_commands")
    .select("id, kind, payload")
    .eq("device_id", deviceId)
    .is("acked_at", null)
    .order("created_at", { ascending: true })
    .limit(50);

  // Mark them delivered (device acks separately when applied).
  if (commands && commands.length > 0) {
    await db.from("device_commands")
      .update({ delivered_at: new Date().toISOString() })
      .in("id", commands.map((c) => c.id));
  }

  return json({
    policy_version: policy?.version ?? null,
    policy: policyChanged ? policy : null,
    commands: commands ?? [],
  });
});
