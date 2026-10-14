// device-sync
// ----------------------------------------------------------------------------
// The device's main polling endpoint. Authenticated by the per-device JWT.
//   * Pulls the active policy for the device's child (if newer than device has).
//   * Pulls pending commands (lock_now, unlink, ...).
//   * Accepts a batch of usage_events to store.
//   * Updates device status (last_seen, battery, foreground app).
//
// The device_id ALWAYS comes from the verified JWT, never from the body.
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

  let body: {
    known_policy_version?: number;
    status?: { battery_pct?: number; foreground_app?: string };
    usage?: Array<{ package_name: string; foreground_s: number; occurred_at: string }>;
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

  // Update live status.
  await db.from("devices").update({
    last_seen_at: new Date().toISOString(),
    battery_pct: body.status?.battery_pct ?? null,
    foreground_app: body.status?.foreground_app ?? null,
  }).eq("id", deviceId);

  // Store usage batch (bounded; validate before trusting).
  if (Array.isArray(body.usage) && body.usage.length > 0) {
    const rows = body.usage.slice(0, 500).map((u) => ({
      device_id: deviceId,
      package_name: String(u.package_name).slice(0, 255),
      foreground_s: Math.max(0, Math.min(86400, Number(u.foreground_s) || 0)),
      occurred_at: u.occurred_at,
    }));
    await db.from("usage_events").insert(rows);
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
