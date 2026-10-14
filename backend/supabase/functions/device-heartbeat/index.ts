// device-heartbeat
// ----------------------------------------------------------------------------
// Lightweight "I'm alive" ping. Cheaper than a full sync; run it frequently so
// the dashboard's "device went silent" detection is timely. Device JWT auth.
import { handleOptions, json } from "../_shared/cors.ts";
import { serviceClient } from "../_shared/supabase.ts";
import { deviceTokenFromRequest, verifyDeviceJwt } from "../_shared/deviceJwt.ts";

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
