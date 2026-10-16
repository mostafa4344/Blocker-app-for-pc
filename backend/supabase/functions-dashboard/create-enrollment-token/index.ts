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
// create-enrollment-token
// ----------------------------------------------------------------------------
// Called by the PARENT web app (Supabase Auth user JWT) to mint a short-lived,
// single-use enrollment token for one child. The RAW token is returned once (to
// render as a QR); only its SHA-256 hash is stored. The child app later presents
// the raw token to `enroll-device`, which hashes + matches + consumes it.
//
// Security:
//   * Parent is authenticated; we check (via their RLS-scoped client) that the
//     child belongs to them before minting.
//   * Raw token is 256 bits of CSPRNG randomness, never persisted.
//   * TTL is short (default 15 min); single-use enforced at consume time.

function randomToken(): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

async function sha256(input: string): Promise<Uint8Array> {
  const buf = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(input));
  return new Uint8Array(buf);
}

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return handleOptions();
  if (req.method !== "POST") return json({ error: "method_not_allowed" }, 405);

  const authHeader = req.headers.get("Authorization") ?? "";
  const userClient = createClient(
    Deno.env.get("SUPABASE_URL")!,
    Deno.env.get("SUPABASE_ANON_KEY")!,
    { global: { headers: { Authorization: authHeader } }, auth: { persistSession: false } },
  );
  const { data: userData, error: uErr } = await userClient.auth.getUser();
  if (uErr || !userData?.user) return json({ error: "unauthenticated" }, 401);

  let body: { child_id?: string };
  try { body = await req.json(); } catch { return json({ error: "bad_json" }, 400); }
  if (!body.child_id) return json({ error: "missing_child_id" }, 400);

  // Ownership check through the parent's RLS-scoped client: can they see this child?
  const { data: child } = await userClient
    .from("children").select("id").eq("id", body.child_id).maybeSingle();
  if (!child) return json({ error: "forbidden" }, 403);

  const ttlMin = Number(Deno.env.get("ENROLLMENT_TOKEN_TTL_MINUTES") ?? "15");
  const raw = randomToken();
  const hash = await sha256(raw);
  const expiresAt = new Date(Date.now() + ttlMin * 60_000).toISOString();

  const db = serviceClient();
  // Optional: invalidate any previous unconsumed tokens for this child.
  await db.from("enrollment_tokens")
    .update({ consumed_at: new Date().toISOString() })
    .eq("child_id", body.child_id).is("consumed_at", null);

  const { error: insErr } = await db.from("enrollment_tokens").insert({
    child_id: body.child_id,
    token_hash: hash,
    expires_at: expiresAt,
  });
  if (insErr) return json({ error: "mint_failed" }, 500);

  // The raw token is returned ONCE. The device needs this + the functions base URL.
  return json({ token: raw, expires_at: expiresAt, ttl_minutes: ttlMin });
});
