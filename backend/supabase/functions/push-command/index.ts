// push-command
// ----------------------------------------------------------------------------
// Called by the PARENT web app (Supabase Auth user JWT) to send a command to a
// device and nudge it awake via FCM. Unlike the device functions, this one
// authenticates a *parent* and checks family ownership.
//
// Why a function (not a plain insert)? Because we also need to send the FCM
// wake signal, and the FCM server key must stay server-side.
//
// Note: the row insert itself is also allowed by RLS (cmd_insert) for the owner,
// but doing it here lets us attach the push in one trusted step.
import { handleOptions, json } from "../_shared/cors.ts";
import { createClient } from "https://esm.sh/@supabase/supabase-js@2.45.4";
import { serviceClient } from "../_shared/supabase.ts";

const ALLOWED_KINDS = new Set([
  "policy_updated", "lock_now", "unlock", "focus_now", "sync", "unlink", "locate_now",
]);

async function sendFcmWake(pushToken: string) {
  const serverKey = Deno.env.get("FCM_SERVER_KEY");
  if (!serverKey || !pushToken) return; // push is best-effort; polling is the fallback
  try {
    await fetch("https://fcm.googleapis.com/fcm/send", {
      method: "POST",
      headers: { "Authorization": `key=${serverKey}`, "Content-Type": "application/json" },
      // Data-only message: a silent nudge to "wake up and call device-sync".
      // Carries NO sensitive payload — the device pulls authoritative state itself.
      body: JSON.stringify({ to: pushToken, data: { type: "sync" }, priority: "high" }),
    });
  } catch { /* ignore; device will poll */ }
}

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return handleOptions();
  if (req.method !== "POST") return json({ error: "method_not_allowed" }, 405);

  // Authenticate the PARENT using their Supabase user JWT (Authorization header).
  const authHeader = req.headers.get("Authorization") ?? "";
  const userClient = createClient(
    Deno.env.get("SUPABASE_URL")!,
    Deno.env.get("SUPABASE_ANON_KEY")!,
    { global: { headers: { Authorization: authHeader } }, auth: { persistSession: false } },
  );
  const { data: userData, error: uErr } = await userClient.auth.getUser();
  if (uErr || !userData?.user) return json({ error: "unauthenticated" }, 401);

  let body: { device_id?: string; kind?: string; payload?: Record<string, unknown> };
  try { body = await req.json(); } catch { return json({ error: "bad_json" }, 400); }
  if (!body.device_id || !body.kind || !ALLOWED_KINDS.has(body.kind)) {
    return json({ error: "bad_request" }, 400);
  }

  // Ownership check via the user's own RLS-scoped client: can they see this device?
  const { data: dev } = await userClient
    .from("devices").select("id").eq("id", body.device_id).maybeSingle();
  if (!dev) return json({ error: "forbidden" }, 403); // RLS hid it => not their device

  // Insert the command + fetch push token with service role.
  const db = serviceClient();
  const { error: insErr } = await db.from("device_commands").insert({
    device_id: body.device_id, kind: body.kind, payload: body.payload ?? {},
  });
  if (insErr) return json({ error: "enqueue_failed" }, 500);

  const { data: full } = await db
    .from("devices").select("push_token").eq("id", body.device_id).maybeSingle();
  if (full?.push_token) await sendFcmWake(full.push_token);

  return json({ ok: true });
});
