# 01 — Architecture

```
            ┌──────────────────────────┐
            │     Parent (browser)     │
            │  parent-web (React/Vite) │
            └─────────────┬────────────┘
                          │ HTTPS (Supabase JS client)
                          │ - Auth (JWT)
                          │ - Postgres REST / RPC (RLS enforced)
                          │ - Realtime subscribe (device status)
                          ▼
   ┌───────────────────────────────────────────────────────┐
   │                      SUPABASE                           │
   │                                                         │
   │  Auth (parents)        Postgres (RLS)      Realtime     │
   │  Edge Functions:                                        │
   │    - enroll-device   (mint/consume device tokens)       │
   │    - device-sync     (device pulls policy, pushes data) │
   │    - push-command    (parent → device command)          │
   │    - device-heartbeat                                   │
   │  Storage (optional: report exports)                     │
   └───────────────┬─────────────────────────┬───────────────┘
                   │ device JWT (per-device)  │ FCM (wake signal)
                   ▼                          ▼
            ┌───────────────────────────────────────┐
            │          Child Android app            │
            │  (Device Owner / Device Admin DPC)     │
            │  - PolicyEngine (local enforcement)    │
            │  - Foreground service + notification   │
            │  - Sync worker (WorkManager)           │
            └───────────────────────────────────────┘
```

## Trust & auth model

There are **two kinds of principals** with very different trust:

1. **Parents** — authenticate with Supabase Auth (email/password, later OAuth). They get a user JWT.
   Postgres **Row Level Security (RLS)** ensures a parent can only read/write rows for *their own
   family*. The web client talks to Postgres directly through Supabase — RLS is the security
   boundary, so the policies in `backend/supabase/policies.sql` are critical and must be reviewed.

2. **Child devices** — are **not** Supabase Auth users. Each device gets its own **device JWT**
   minted by the `enroll-device` Edge Function after it presents a valid, short-lived enrollment
   token. Devices never use the parent's credentials. Devices talk only to Edge Functions (service
   role, server-side checks), never directly to Postgres. This keeps the device's blast radius tiny:
   a compromised device can only touch its own rows, via functions that re-check ownership.

> Why not give the device a Supabase Auth account? Because then a child could potentially extract the
> refresh token and get broad API access. A narrow, per-device JWT scoped to one `device_id` and
> validated by Edge Functions is much safer.

## Data flow: setting a policy

1. Parent edits policy on website → `UPDATE policies ...` via Supabase (RLS checks parent owns child).
2. A Postgres trigger writes a row into `device_commands` (`kind = 'policy_updated'`) and/or
   Realtime broadcasts on a channel the device-sync path watches.
3. Backend sends an **FCM data message** to the device (a silent "wake up and sync" nudge — carries
   no sensitive payload).
4. Child app's sync worker calls `device-sync` Edge Function with its device JWT → gets latest policy
   → `PolicyEngine` applies it locally.
5. Device acknowledges; `device_commands` row marked delivered/acked; website shows "applied".

**Offline-first:** the device caches the last policy and keeps enforcing it with no network. FCM is
only a latency optimization; WorkManager also polls on a schedule as a fallback.

## Why device-side enforcement

At hundreds of millions of devices, you cannot have the backend in the loop for every app launch.
The device holds the policy and enforces locally in real time. The backend is for **configuration,
reporting, and commands** — not per-event decisions. This keeps backend cost and latency bounded.

## Component responsibilities

### parent-web
- Auth screens, family/child management, policy editor, dashboards.
- Pure client of Supabase. No secrets beyond the public anon key.

### backend (Supabase)
- Source of truth for families, children, devices, policies, usage, commands.
- RLS for parents. Edge Functions for devices + privileged operations.
- Realtime for live device status on the dashboard.

### child-app-android
- Enrollment & disclosure/consent flow.
- `PolicyEngine`: app blocking, time limits, web filter, lock.
- Foreground service + **mandatory persistent notification** (compliance).
- Device Owner/Admin for removal resistance.
- Sync worker + FCM receiver.

## Key design decisions (and their trade-offs)

| Decision | Why | Trade-off |
|---|---|---|
| Website for parents, no parent app in v1 | Smaller scope, faster to ship, cross-platform | Parents need a browser; push to parent is email/web only |
| Per-device JWT, devices talk only to Edge Functions | Limits blast radius of a compromised child device | More Edge Function code than "just use the DB" |
| Device-side enforcement | Scale + offline | Policy changes have propagation latency |
| Supabase | Fast to build, owner will host it, batteries included | Vendor-specific; plan an exit path if scaling demands it |
| Device Owner for uninstall protection | Strongest Android has | Requires factory-reset/new-device provisioning for full strength |

See `docs/06-DATA-MODEL.md` for tables and `docs/02-SECURITY.md` for the threat model.
