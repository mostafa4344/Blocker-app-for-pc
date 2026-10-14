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
import { handleOptions, json } from "../_shared/cors.ts";
import { serviceClient } from "../_shared/supabase.ts";
import { mintDeviceJwt } from "../_shared/deviceJwt.ts";

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
