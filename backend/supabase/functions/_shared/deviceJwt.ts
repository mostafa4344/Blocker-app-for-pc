// Per-device JWT helpers (HS256). The secret lives only in function secrets.
//
// A device JWT carries exactly one meaningful claim: `device_id`. Every function
// that accepts a device re-derives the device_id from this verified token and
// NEVER trusts a device_id sent in the request body.
//
// Uses `djwt` (Deno). Pin the version when you deploy.
import { create, verify, getNumericDate } from "https://deno.land/x/djwt@v3.0.2/mod.ts";

const encoder = new TextEncoder();

async function key(): Promise<CryptoKey> {
  const secret = Deno.env.get("DEVICE_JWT_SECRET");
  if (!secret) throw new Error("DEVICE_JWT_SECRET not set");
  return await crypto.subtle.importKey(
    "raw",
    encoder.encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign", "verify"],
  );
}

export async function mintDeviceJwt(deviceId: string): Promise<string> {
  const ttlDays = Number(Deno.env.get("DEVICE_JWT_TTL_DAYS") ?? "30");
  return await create(
    { alg: "HS256", typ: "JWT" },
    {
      sub: deviceId,
      device_id: deviceId,
      scope: "device",
      exp: getNumericDate(ttlDays * 24 * 60 * 60),
    },
    await key(),
  );
}

/** Returns the device_id from a valid token, or null if invalid/expired. */
export async function verifyDeviceJwt(token: string): Promise<string | null> {
  try {
    const payload = await verify(token, await key());
    if (payload.scope !== "device" || typeof payload.device_id !== "string") return null;
    return payload.device_id;
  } catch {
    return null;
  }
}

/** Pull the device JWT from the `x-device-jwt` header. */
export function deviceTokenFromRequest(req: Request): string | null {
  return req.headers.get("x-device-jwt");
}
