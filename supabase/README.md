# Supabase setup for Content Guard

## One step (recommended)

Open your Supabase project → **SQL editor** → paste the contents of
[`setup.sql`](setup.sql) → **Run**. It is safe to re-run.

That creates the `accounts` and `codes` tables (RLS on, no direct anon access),
the public RPCs the app calls with the publishable key
(`register_device`, `get_license`, `redeem_code`), and admin RPCs for a payment
webhook (`admin_grant_premium`, `admin_create_code`, service-role only). It
inserts no activation codes (anything in this public repo is public).

Since `20261030000000_signed_licenses.sql` the app calls `license_register`,
`license_check` and `license_redeem`. Every answer carries a license token
signed with a key kept in Supabase Vault; the app only trusts that signature.

The desktop app already points at this project (URL + publishable key are the
built-in defaults in `crates/agent/src/license.rs`), so nothing else is needed
for licensing to work once the SQL has run.

## Security model

- The app uses only the **publishable (anon)** key and can call only the three
  public RPCs. Tables have RLS enabled with **no** anon/authenticated policies,
  so the anon key can never read or write rows directly (verified).
- Premium is granted only by `admin_grant_premium` / `admin_create_code`, which
  are granted to **service_role** only — call them from a trusted server / the
  payment webhook, never from the client.

## Granting premium after a payment

From a server (Edge Function / webhook) using the **service-role** key:

```sql
select admin_grant_premium('<install_id>', 30);   -- 30 days; null = lifetime
-- or hand out a one-time code:
select admin_create_code('GIFT-ABC123', 365, 'promo');
```

The app reads the device's `install_id` from **Settings → (advanced)**; a webhook
gets it from the order you capture at checkout.

## Files

- `setup.sql` — the single file to run (tables + RPCs + admin + test codes).
- `migrations/` + `seed.sql` — the same thing split for `supabase db push` / CLI.
