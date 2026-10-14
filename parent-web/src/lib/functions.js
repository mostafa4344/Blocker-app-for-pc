import { supabase, FUNCTIONS_URL } from "./supabase.js";

/**
 * Call a Supabase Edge Function with the current parent's auth token.
 * Used for operations that must run server-side (minting enrollment tokens,
 * sending device commands + FCM wake) where the service role / FCM key are needed.
 */
export async function callFunction(name, body) {
  const { data: { session } } = await supabase.auth.getSession();
  if (!session) throw new Error("Not signed in");

  const res = await fetch(`${FUNCTIONS_URL}/${name}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${session.access_token}`,
    },
    body: JSON.stringify(body ?? {}),
  });

  const json = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(json.error || `Function ${name} failed (${res.status})`);
  return json;
}
