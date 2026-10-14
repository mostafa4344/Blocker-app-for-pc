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
import { handleOptions, json } from "../_shared/cors.ts";
import { createClient } from "https://esm.sh/@supabase/supabase-js@2.45.4";
import { serviceClient } from "../_shared/supabase.ts";

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
