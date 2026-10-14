// Service-role Supabase client for Edge Functions. Bypasses RLS, so every
// function MUST do its own ownership checks (device JWT -> device_id) before
// reading/writing rows. Never expose this client or its key to any client app.
import { createClient } from "https://esm.sh/@supabase/supabase-js@2.45.4";

export function serviceClient() {
  const url = Deno.env.get("SUPABASE_URL");
  const serviceKey = Deno.env.get("SUPABASE_SERVICE_ROLE_KEY");
  if (!url || !serviceKey) throw new Error("Supabase service env not set");
  return createClient(url, serviceKey, {
    auth: { persistSession: false, autoRefreshToken: false },
  });
}
