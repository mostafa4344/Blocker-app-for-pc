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
// enroll-device
// ----------------------------------------------------------------------------
// Called by the child Android app during setup with a raw enrollment token
// (from the QR the parent sees on the website) and basic device info.
//
// Flow:
//   1. Hash the raw token (sha256) and look up a matching, unexpired, unconsumed
//      enrollment_tokens row.
//   2. Atomically consume it: create a `devices` row bound to that child, mark
//      the token consumed.
//   3. Mint a per-device JWT scoped to the new device_id and return it.
//
// Security notes:
//   * The raw token never persists server-side; only its hash is stored.
//   * Single-use + short TTL enforced in SQL (consumed_at / expires_at).
//   * The device JWT is the ONLY credential the device keeps. It can reach only
//     its own device's rows, via the other functions.

async function sha256(input: string): Promise<Uint8Array> {
  const buf = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(input));
  return new Uint8Array(buf);
}

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return handleOptions();
  if (req.method !== "POST") return json({ error: "method_not_allowed" }, 405);

  let body: { token?: string; device?: Record<string, unknown> };
  try {
    body = await req.json();
  } catch {
    return json({ error: "bad_json" }, 400);
  }
  const rawToken = body?.token;
  if (!rawToken || typeof rawToken !== "string") return json({ error: "missing_token" }, 400);

  const db = serviceClient();
  const tokenHash = await sha256(rawToken);

  // 1. find a valid token
  const { data: tok, error: tErr } = await db
    .from("enrollment_tokens")
    .select("id, child_id, expires_at, consumed_at")
    .eq("token_hash", tokenHash)
    .is("consumed_at", null)
    .gt("expires_at", new Date().toISOString())
    .maybeSingle();

  if (tErr) return json({ error: "server_error" }, 500);
  if (!tok) return json({ error: "invalid_or_expired_token" }, 401);

  // 2. create the device
  const dev = body.device ?? {};
  const { data: device, error: dErr } = await db
    .from("devices")
    .insert({
      child_id: tok.child_id,
      label: (dev.label as string) ?? "Phone",
      model: (dev.model as string) ?? null,
      os_version: (dev.os_version as string) ?? null,
      management_mode: (dev.management_mode as string) ?? "unprovisioned",
      push_token: (dev.push_token as string) ?? null,
      last_seen_at: new Date().toISOString(),
    })
    .select("id")
    .single();

  if (dErr || !device) return json({ error: "device_create_failed" }, 500);

  // 3. consume the token (bind to this device). Guard against double-consume.
  const { error: cErr } = await db
    .from("enrollment_tokens")
    .update({ consumed_at: new Date().toISOString(), device_id: device.id })
    .eq("id", tok.id)
    .is("consumed_at", null);

  if (cErr) {
    // best-effort rollback of the device row
    await db.from("devices").delete().eq("id", device.id);
    return json({ error: "token_already_used" }, 409);
  }

  const deviceJwt = await mintDeviceJwt(device.id);
  return json({ device_id: device.id, device_jwt: deviceJwt });
});
