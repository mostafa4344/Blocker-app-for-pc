// Content Guard — payment webhook (Supabase Edge Function, Deno).
//
// Deploy:   supabase functions deploy payment-webhook --no-verify-jwt
// Secrets:  supabase secrets set WEBHOOK_SECRET=<random>  SERVICE_ROLE_KEY=<service-role key>
//           (SUPABASE_URL is provided automatically.)
//
// Point your payment provider's webhook (Stripe / bKash / Nagad / etc.) at:
//   https://<project>.functions.supabase.co/payment-webhook
// After verifying the payment, it grants premium via the service-role RPC
// `admin_grant_premium(install_id, days)`. Keep the real provider-signature
// verification for your chosen provider; the shared-secret check below is the
// minimum and is provider-agnostic.

import { createClient } from "https://esm.sh/@supabase/supabase-js@2.45.4";

const SUPABASE_URL = Deno.env.get("SUPABASE_URL")!;
const SERVICE_ROLE_KEY = Deno.env.get("SERVICE_ROLE_KEY")!;
const WEBHOOK_SECRET = Deno.env.get("WEBHOOK_SECRET")!;

Deno.serve(async (req) => {
  if (req.method !== "POST") return new Response("method", { status: 405 });

  // Minimal auth: a shared secret header. Replace/augment with your provider's
  // signature verification (e.g. Stripe-Signature) for production.
  if (req.headers.get("x-webhook-secret") !== WEBHOOK_SECRET) {
    return new Response("unauthorized", { status: 401 });
  }

  let body: { install_id?: string; days?: number | null };
  try {
    body = await req.json();
  } catch {
    return new Response("bad json", { status: 400 });
  }
  const installId = (body.install_id ?? "").trim();
  if (installId.length < 16) return new Response("install_id required", { status: 400 });
  // days: a number of days, or null for a lifetime/one-time purchase.
  const days = body.days === null || body.days === undefined ? null : Number(body.days);

  const admin = createClient(SUPABASE_URL, SERVICE_ROLE_KEY);
  const { data, error } = await admin.rpc("admin_grant_premium", {
    p_install_id: installId,
    p_days: days,
  });
  if (error) return new Response(JSON.stringify({ error: error.message }), { status: 400 });
  return new Response(JSON.stringify({ ok: true, license: data }), {
    headers: { "content-type": "application/json" },
  });
});
