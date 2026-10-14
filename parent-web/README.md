# Parent Web Dashboard

The website parents log into to see and manage their child's device. React + Vite + Supabase JS.

## Status
**Skeleton.** Auth + Supabase client + app shell are scaffolded. Family/child management, the policy
editor, enrollment-QR screen, live status, and reports are stubbed — see `../docs/05-ROADMAP.md`.

## Run
```bash
cp .env.example .env    # fill in your Supabase URL + anon key
npm install
npm run dev
```

## Env
Only the **public** Supabase URL and anon key belong here. The service-role key and FCM key must
NEVER appear in this app — they live only in Edge Function secrets. See `../docs/02-SECURITY.md`.

## Security reminder
The web app talks to Postgres directly through Supabase; **RLS** (`../backend/supabase/policies.sql`)
is what keeps one family's data private from another. The client cannot be trusted to enforce access;
the database does.

## Structure (planned)
```
src/
├── main.jsx              # entry
├── App.jsx               # router + auth gate
├── lib/supabase.js       # Supabase client (anon key)
├── context/AuthContext.jsx
├── pages/
│   ├── Login.jsx
│   ├── Dashboard.jsx     # list children + device status
│   ├── ChildDetail.jsx   # policy editor + reports
│   └── EnrollDevice.jsx  # shows enrollment QR
└── components/
```
