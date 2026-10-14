# 06 — Data Model

Postgres (Supabase). Authoritative DDL is in `backend/supabase/schema.sql`; RLS in
`backend/supabase/policies.sql`. This doc explains the *why*.

## Entities

```
auth.users (Supabase)            families
     │ 1                              │ 1
     │                                │
     ▼ *                              ▼ *
family_members ───────────────▶ children
  (role: owner/guardian)             │ 1
                                     │
                                     ▼ *
                                  devices
                                     │ 1
            ┌────────────────────────┼───────────────────────┐
            ▼ *                      ▼ *                       ▼ *
        policies            device_commands            usage_events / heartbeats / locations
     (1 active per child)   (parent→device queue)        (device→backend telemetry)

enrollment_tokens  (short-lived, ties a device to a child during setup)
```

## Tables (summary)

| Table | Purpose | Written by |
|---|---|---|
| `families` | A household/account. | Parent |
| `family_members` | Links `auth.users` ↔ `families` with a role (`owner`, `guardian`). Enables two parents. | Parent |
| `children` | A child profile (name, birth year for age-appropriate defaults). | Parent |
| `devices` | A physical device bound to a child. Holds status, `revoked_at`, push token. | Parent (create/revoke) + device (status) |
| `enrollment_tokens` | Short-lived single-use tokens to bind a device to a child at setup. | Backend (mint/consume) |
| `policies` | The rules for a child (JSON policy doc + structured columns). One active per child. | Parent |
| `device_commands` | Queue of parent→device commands (lock now, sync, unlink). | Parent / triggers |
| `usage_events` | App usage telemetry (package, foreground seconds, bucketed). | Device |
| `heartbeats` | Periodic "I'm alive" + quick status. Drives "device went silent". | Device |
| `locations` | Opt-in coarse location points, short retention. | Device |

## Why a `policies` JSON column *and* structured columns
The policy has a stable core (screen-time minutes, bedtime window, location on/off) that we query and
validate → those are **columns**. The long tail (per-app rules, web categories) evolves fast → kept
in a validated **`rules` jsonb**. This balances query-ability with flexibility and avoids a migration
for every new rule type.

## Privacy-driven retention
- `usage_events`: aggregate to daily summaries after N days, then drop raw rows.
- `locations`: keep only a short rolling window (e.g. 7–30 days), then delete.
- No message/call/keystroke tables exist **by design** — there is nowhere to store such data.

## Indexing notes (for scale, phase 5)
- `usage_events (device_id, occurred_at)` — reporting queries.
- `heartbeats (device_id, created_at desc)` — latest-status lookups; consider keeping only latest +
  a short history table.
- `devices (child_id)`, `children (family_id)`, `family_members (user_id)` — RLS join paths; keep
  these indexed because RLS checks run on every query.
- At very large scale, partition `usage_events` / `locations` by time.

## RLS join path (the security-critical bit)
A parent may touch a row iff it chains back to a family they belong to:
```
row → child → family → family_members(user_id = auth.uid())
```
Every policy in `policies.sql` encodes exactly this chain. Devices do **not** use RLS — they go
through Edge Functions that re-derive `device_id` from the verified device JWT.
