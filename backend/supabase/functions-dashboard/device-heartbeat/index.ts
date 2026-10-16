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
// device-heartbeat
// ----------------------------------------------------------------------------
// Lightweight "I'm alive" ping. Cheaper than a full sync; run it frequently so
// the dashboard's "device went silent" detection is timely. Device JWT auth.

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return handleOptions();
  if (req.method !== "POST") return json({ error: "method_not_allowed" }, 405);

  const token = deviceTokenFromRequest(req);
  if (!token) return json({ error: "missing_device_jwt" }, 401);
  const deviceId = await verifyDeviceJwt(token);
  if (!deviceId) return json({ error: "invalid_device_jwt" }, 401);

  let body: { battery_pct?: number; foreground_app?: string; policy_version?: number } = {};
  try {
    if (req.headers.get("content-length") !== "0") body = await req.json();
  } catch { /* ignore, heartbeat can be empty */ }

  const db = serviceClient();

  const { data: device } = await db
    .from("devices").select("id, revoked_at").eq("id", deviceId).maybeSingle();
  if (!device) return json({ error: "unknown_device" }, 404);
  if (device.revoked_at) return json({ error: "revoked", action: "self_disable" }, 403);

  const now = new Date().toISOString();
  await db.from("devices").update({
    last_seen_at: now,
    battery_pct: body.battery_pct ?? null,
    foreground_app: body.foreground_app ?? null,
  }).eq("id", deviceId);

  await db.from("heartbeats").insert({
    device_id: deviceId,
    battery_pct: body.battery_pct ?? null,
    foreground_app: body.foreground_app ?? null,
    policy_version: body.policy_version ?? null,
  });

  return json({ ok: true, server_time: now });
});
