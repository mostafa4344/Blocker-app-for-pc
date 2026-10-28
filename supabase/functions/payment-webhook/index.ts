// HTL Content Guard — payment webhook (Supabase Edge Function, Deno).
//
// Grants Premium automatically when a payment succeeds. Two entry points:
//
//   POST .../payment-webhook?provider=paddle
//       Paddle Billing notifications. Verified with the `Paddle-Signature`
//       header (HMAC-SHA256 of "<ts>:<raw body>" with PADDLE_WEBHOOK_SECRET).
//       Handles `transaction.completed`. The checkout must pass
//       customData: { install_id } (the purchase page does this).
//
//   POST .../payment-webhook?provider=generic
//       Any other gateway (SSLCommerz, bKash, a manual tool…) that can send
//       JSON { event_id, install_id, product } with header
//       `x-webhook-secret: <WEBHOOK_SECRET>`.
//
// Products -> days come from PRODUCT_DAYS, e.g.
//   {"pri_01monthly":30,"pri_01yearly":365,"pri_01lifetime":null,
//    "monthly":30,"yearly":365,"lifetime":null}
// (Paddle price ids for Paddle; plain names for the generic path.)
//
// Deploy:  supabase functions deploy payment-webhook --no-verify-jwt
// Secrets: supabase secrets set PADDLE_WEBHOOK_SECRET=... WEBHOOK_SECRET=... PRODUCT_DAYS='{...}'
//          (SUPABASE_URL and SUPABASE_SERVICE_ROLE_KEY are provided automatically.)
//
// Every grant goes through grant_premium_for_payment(), which is idempotent per
// event id, so retries from the gateway never grant twice.

import { createClient } from "https://esm.sh/@supabase/supabase-js@2.45.4";

const enc = new TextEncoder();

/** Constant-time comparison of two hex strings. */
export function safeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}

export async function hmacHex(secret: string, message: string): Promise<string> {
  const key = await crypto.subtle.importKey("raw", enc.encode(secret), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  const sig = await crypto.subtle.sign("HMAC", key, enc.encode(message));
  return [...new Uint8Array(sig)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/**
 * Verifies a Paddle Billing `Paddle-Signature` header ("ts=...;h1=...").
 * Rejects signatures older than `maxAgeSec` to stop replays.
 */
export async function verifyPaddle(
  header: string | null,
  rawBody: string,
  secret: string,
  nowSec = Math.floor(Date.now() / 1000),
  maxAgeSec = 300,
): Promise<boolean> {
  if (!header || !secret) return false;
  const parts = Object.fromEntries(header.split(";").map((p) => p.split("=", 2) as [string, string]));
  const ts = Number(parts.ts);
  const h1 = parts.h1 ?? "";
  if (!Number.isFinite(ts) || Math.abs(nowSec - ts) > maxAgeSec || !h1) return false;
  return safeEqual(await hmacHex(secret, `${parts.ts}:${rawBody}`), h1);
}

/** Days for a product id/name; `undefined` = unknown product. null = lifetime. */
export function daysFor(product: string, table: Record<string, number | null>): number | null | undefined {
  return Object.prototype.hasOwnProperty.call(table, product) ? table[product] : undefined;
}

const json = (status: number, body: unknown) =>
  new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });

async function grant(p: {
  eventId: string; provider: string; installId: string; days: number | null;
  product: string; amount?: string; currency?: string; raw?: unknown;
}) {
  const admin = createClient(Deno.env.get("SUPABASE_URL")!, Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!);
  return await admin.rpc("grant_premium_for_payment", {
    p_event_id: p.eventId, p_provider: p.provider, p_install_id: p.installId, p_days: p.days,
    p_product: p.product, p_amount: p.amount ?? "", p_currency: p.currency ?? "", p_raw: p.raw ?? null,
  });
}

export async function handle(req: Request): Promise<Response> {
  if (req.method !== "POST") return json(405, { error: "method" });
  const provider = new URL(req.url).searchParams.get("provider") ?? "generic";
  const raw = await req.text();
  let table: Record<string, number | null> = {};
  try {
    table = JSON.parse(Deno.env.get("PRODUCT_DAYS") ?? "{}");
  } catch {
    return json(500, { error: "PRODUCT_DAYS is not valid JSON" });
  }

  if (provider === "paddle") {
    if (!(await verifyPaddle(req.headers.get("paddle-signature"), raw, Deno.env.get("PADDLE_WEBHOOK_SECRET") ?? ""))) {
      return json(401, { error: "bad signature" });
    }
    const ev = JSON.parse(raw);
    if (ev.event_type !== "transaction.completed") return json(200, { ignored: ev.event_type });
    const d = ev.data ?? {};
    const installId = String(d.custom_data?.install_id ?? "").trim();
    const priceId = String(d.items?.[0]?.price?.id ?? "");
    const days = daysFor(priceId, table);
    if (installId.length < 16 || days === undefined) {
      // Acknowledge (so Paddle stops retrying) but flag for the admin.
      console.error("unmatched paddle payment", ev.event_id, priceId, installId.slice(0, 8));
      return json(200, { warning: "unmatched", event_id: ev.event_id });
    }
    const { data, error } = await grant({
      eventId: `paddle:${ev.event_id}`, provider, installId, days, product: priceId,
      amount: String(d.details?.totals?.grand_total ?? ""), currency: String(d.currency_code ?? ""), raw: ev,
    });
    if (error) return json(500, { error: error.message }); // Paddle retries later
    return json(200, { ok: true, license: data });
  }

  // Generic gateways: shared secret.
  const secret = Deno.env.get("WEBHOOK_SECRET") ?? "";
  if (!secret || !safeEqual(req.headers.get("x-webhook-secret") ?? "", secret)) return json(401, { error: "unauthorized" });
  let body: { event_id?: string; install_id?: string; product?: string; amount?: string; currency?: string };
  try {
    body = JSON.parse(raw);
  } catch {
    return json(400, { error: "bad json" });
  }
  const installId = (body.install_id ?? "").trim();
  const days = daysFor(body.product ?? "", table);
  if (!body.event_id || installId.length < 16 || days === undefined) {
    return json(400, { error: "event_id, install_id and a known product are required" });
  }
  const { data, error } = await grant({
    eventId: `${provider}:${body.event_id}`, provider, installId, days, product: body.product!,
    amount: body.amount, currency: body.currency, raw: body,
  });
  if (error) return json(400, { error: error.message });
  return json(200, { ok: true, license: data });
}

if (import.meta.main) Deno.serve(handle);
