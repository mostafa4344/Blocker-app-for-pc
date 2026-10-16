# 09 — Setup for YOUR Supabase project

Filled in for the project you created.

| Thing | Value |
|---|---|
| Project URL | `https://eihaavfsupquzdxcixjb.supabase.co` |
| Project ref | `eihaavfsupquzdxcixjb` |
| Publishable (anon) key | `sb_publishable_swMjJ38v3rCDAL-MffNfBw_z_f2ULBu` — safe in the browser/app |
| Functions base URL | `https://eihaavfsupquzdxcixjb.supabase.co/functions/v1` |

The web `.env` and the Android Gradle default are already wired to these.

> 🔐 The **service-role key** and **database password** are secrets. Use them only
> in the Supabase dashboard / CLI on your own machine. Do not paste them into chat
> or commit them.

---

## Step 1 — Create the tables + security rules (owner-only, ~1 min)

1. Open the project → **SQL Editor** → **New query**.
2. Paste the entire contents of **`backend/supabase/setup_all.sql`** and click **Run**.
   (That file = `schema.sql` + `policies.sql` combined.)
3. You should see "Success". This creates all tables, indexes, triggers, the
   `create_family` RPC, and Row Level Security.

## Step 2 — Turn on Realtime for live status (optional, 20 s)

Database → **Replication** (or Realtime) → enable for the `devices` table. This
makes the dashboard update the moment a device checks in.

## Step 3 — Deploy the Edge Functions (owner-only, needs the CLI)

On your own computer:

```bash
npm install -g supabase            # if not installed
supabase login                     # opens browser
supabase link --project-ref eihaavfsupquzdxcixjb

# From the repo root:
supabase functions deploy enroll-device create-enrollment-token \
  device-sync device-heartbeat push-command --project-ref eihaavfsupquzdxcixjb
```

Then set the function secrets (these stay server-side, never in any app):

```bash
supabase secrets set \
  DEVICE_JWT_SECRET="$(openssl rand -hex 32)" \
  ENROLLMENT_TOKEN_TTL_MINUTES=15 \
  DEVICE_JWT_TTL_DAYS=30 \
  --project-ref eihaavfsupquzdxcixjb
# FCM_SERVER_KEY is only needed once you add Firebase (push). Skip for now.
```

> `SUPABASE_URL`, `SUPABASE_ANON_KEY`, and `SUPABASE_SERVICE_ROLE_KEY` are provided
> to Edge Functions automatically by Supabase — you do not set those yourself.

## Step 4 — Run the parent website

```bash
cd parent-web
npm install
npm run dev        # opens http://localhost:5173
```

`.env` is already set. Sign up → add a child → open the child → **+ Enroll device**
to see the QR. (The QR only works end-to-end after Step 3, because it calls
`create-enrollment-token`.)

## Step 5 — Build the Android app

Open `child-app-android/` in Android Studio and Run on a test phone. It already
points at your functions URL. (Full device testing + Firebase/FCM come later —
see `docs/05-ROADMAP.md`.)

---

## Quick self-check after Step 1

In the SQL editor, run:
```sql
select table_name from information_schema.tables
where table_schema = 'public' order by 1;
```
You should see: `children, device_commands, devices, enrollment_tokens, families,
family_members, heartbeats, locations, policies, usage_events`.
