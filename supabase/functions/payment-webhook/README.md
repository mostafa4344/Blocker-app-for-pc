# Payment webhook (template)

A provider-agnostic Supabase Edge Function that grants Premium after a payment.

1. `supabase functions deploy payment-webhook --no-verify-jwt`
2. `supabase secrets set WEBHOOK_SECRET=<random> SERVICE_ROLE_KEY=<service-role key>`
3. In your payment provider (Stripe / bKash / Nagad / card gateway), set the
   success webhook URL to `https://<project>.functions.supabase.co/payment-webhook`
   and send `{ "install_id": "<device id>", "days": 30 }` (or `"days": null` for
   lifetime). Capture the buyer's **device id** at checkout — the app shows it in
   **Settings → (advanced)**.
4. Replace the shared-secret check with your provider's real signature
   verification before going live.

The function calls the `admin_grant_premium` RPC (service-role only) created by
`../setup.sql`.
