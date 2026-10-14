# Backend (Supabase)

All backend code lives here so it can be transferred into your own Supabase project later.

## What's here

```
backend/
├── README.md
├── .env.example                  # env vars the Edge Functions need (copy to .env, never commit)
└── supabase/
    ├── schema.sql                # tables, indexes, triggers, helper functions
    ├── policies.sql              # Row Level Security (SECURITY-CRITICAL — review & test)
    ├── seed.sql                  # optional local dev seed
    └── functions/                # Edge Functions (Deno/TypeScript)
        ├── _shared/              # cors + device-JWT helpers
        ├── enroll-device/        # consume enrollment token -> mint device JWT
        ├── device-sync/          # device pulls policy + pending commands, pushes usage
        ├── device-heartbeat/     # lightweight "I'm alive" + status
        └── push-command/         # parent -> device command (sends FCM wake)
```

## Setup (when you move to your own Supabase)

1. Create a Supabase project. Note the project URL and keys.
2. In the SQL editor, run in order: **`schema.sql`**, then **`policies.sql`**. (Optional: `seed.sql`.)
3. Install the Supabase CLI and link: `supabase link --project-ref <ref>`.
4. Set function secrets (do NOT hardcode):
   ```
   supabase secrets set DEVICE_JWT_SECRET=... FCM_SERVER_KEY=... SUPABASE_SERVICE_ROLE_KEY=...
   ```
5. Deploy functions: `supabase functions deploy enroll-device device-sync device-heartbeat push-command`.
6. Verify RLS with the test checklist at the top of `policies.sql` **before** any real data.

## Two auth lanes (important)

- **Parents** → Supabase Auth + RLS. The web app uses the public anon key.
- **Devices** → a per-device JWT minted by `enroll-device`, verified by the other functions. Devices
  never get the anon key's data access path; they only call Edge Functions. The **service role key**
  and **DEVICE_JWT_SECRET** live only in function secrets — never in the web or Android app.

## Notes
- These functions are **skeletons** with the real control flow and security checks sketched, but they
  are not yet hardened or deployed. Treat them as a starting point, not finished endpoints.
- Rate limiting, structured logging, and input validation (zod) should be added before launch.
