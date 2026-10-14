# Supabase migration

The temporary backend in `../backend` (Node, JSON file) implements the API the
agent uses. Moving it to Supabase:

1. `supabase db push` — applies `migrations/` (tables, RLS, helper functions).
2. Port `backend/src/app.mjs` handlers to one Edge Function (`functions/api`).
   Replace `JsonStore` calls with `supabase-js` using the service-role key:
   | JsonStore | Supabase |
   |---|---|
   | `insertDevice` / `getDevice` / `updateDevice` | `from('devices').insert/select/update` |
   | `insertRequest` / `getRequest` / `getRequestByTokenHash` / `updateRequest` | `from('requests')…` |
   | `insertOutbox` | `from('outbox').insert` |
   | `effectiveStatus(r)` | `request_status(r)` SQL function |
   | `checkSilentDevices()` | `silent_devices()` + pg_cron every minute |
3. Send emails from the `outbox` table (Resend/SMTP) in a scheduled function.
4. Set the agent's sync server URL to `https://<project>.supabase.co/functions/v1/api`
   (the paths `/v1/devices/...` and `/approve/...` stay the same).
5. Copy existing data with `node backend/scripts/export-sql.mjs > seed.sql` if needed.
