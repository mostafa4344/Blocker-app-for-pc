# 02 — Security Design & Threat Model

Security here has two faces:
1. **Protecting families' data** (a breach exposes children's locations and habits — extremely sensitive).
2. **Keeping the system from being abusable** (it must not become a tool for covert surveillance).

Both are first-class goals.

## Assets we protect

- Children's location, app usage, schedules, device identifiers.
- Parent credentials and account.
- Enrollment tokens and device JWTs.
- The integrity of policy enforcement on the device.

## Principals & trust levels

| Principal | Trust | Mechanism |
|---|---|---|
| Parent | Authenticated user, scoped to own family | Supabase Auth JWT + Postgres RLS |
| Child device | Semi-trusted, scoped to one device | Per-device JWT, validated by Edge Functions |
| Attacker with physical access to child phone | Hostile, may be the child | Device Owner, secure storage, server re-checks |
| Public internet | Hostile | TLS, no secrets in client, rate limits |

## Core controls

### Row Level Security (RLS) — the main boundary for parents
The parent web app talks to Postgres directly. **RLS is what stops parent A from reading parent B's
family.** Every table with family data has policies keyed off `auth.uid()` → `family_members`. These
live in `backend/supabase/policies.sql` and must be reviewed line-by-line and tested. A missing or
loose policy = data breach. There is a test checklist in that file's header.

### Devices never touch Postgres directly
Devices only call Edge Functions, authenticated by a per-device JWT. Functions use the service role
but **re-derive the device's `device_id` from the verified JWT** and only ever read/write that
device's own rows. Never trust an `id` sent in the request body.

### Enrollment tokens
- Short-lived (e.g. 15 min), single-use, random (≥128 bits), bound to one child profile.
- Consumed atomically by `enroll-device`; a used/expired token is rejected.
- Displayed as a QR on the parent website only to an authenticated parent.

### Device JWT
- Minted on successful enrollment, scoped to `device_id`, with a moderate TTL + rotation.
- Stored on-device in Android **EncryptedSharedPreferences** / Keystore-backed storage.
- Revocable server-side (a `devices.revoked_at` column; functions reject revoked devices) so a
  parent can "unlink" a device.

### Transport & data at rest
- TLS everywhere (Supabase default).
- Postgres encrypted at rest (Supabase/managed).
- Minimize what we collect (see compliance). Location stored only if the parent enables it; retained
  for a bounded window, then aggregated/deleted.

### Secrets
- No service-role key or FCM server key ever ships in the web or Android app. They live only in Edge
  Function environment config. See `.env.example` files.

## Threat model (STRIDE-ish, abbreviated)

| Threat | Example | Mitigation |
|---|---|---|
| Parent A reads Parent B's data | Guessing child IDs | RLS keyed to `auth.uid()`; no cross-family joins |
| Child extracts device JWT and calls API | Rooted phone | JWT scoped to own `device_id`; functions re-check; revocable; nothing sensitive reachable beyond own device |
| Child uninstalls/disables the app | Factory reset, recovery | Device Owner resists; parent sees "device went silent"; honest limits documented |
| Stolen enrollment QR | Shoulder-surf the dashboard | Short-lived, single-use, bound to one child |
| Mass scraping via Edge Functions | Script hits endpoints | Rate limiting + device JWT required + per-device quotas |
| Insider / DB dump | Supabase breach | Encryption at rest, data minimization, short retention, no message/call content ever stored |
| Product abused as stalkerware | Repurposed for a spouse | Persistent notification is non-removable by design; marketing + onboarding enforce parental-only use; see compliance |

## "Device went silent" is a feature
Because we can't guarantee the app is unremovable, the **parent dashboard treats loss of heartbeat as
a visible event** ("Alex's phone hasn't checked in since 3:12pm"). That's more honest and more useful
than pretending removal is impossible, and it's the right security posture: detect, don't promise
prevention.

## What we deliberately do NOT build
No message/call/keystroke capture, no hidden mode, no remote mic/camera. These are both a policy ban
and a security/ethics landmine. See `docs/03-PLAY-STORE-COMPLIANCE.md`.

## Pre-launch security checklist (must all pass)
- [ ] Every family table has RLS; verified with an automated test that parent B cannot read parent A.
- [ ] Edge Functions re-derive device identity from JWT, never from request body.
- [ ] Enrollment tokens are single-use, short-lived, random, bound.
- [ ] Device JWTs are revocable and rotated.
- [ ] No secrets in client bundles (grep the built web bundle + APK).
- [ ] Rate limiting on all Edge Functions.
- [ ] Data retention job deletes old location/usage rows.
- [ ] Third-party security review / pen-test before public launch.
