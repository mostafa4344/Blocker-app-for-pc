# 05 — Roadmap & Status

## Progress log

- **v0.1** — foundation/scaffold (docs, schema, RLS, skeletons). See below.
- **v0.2** — Phase 1 in progress:
  - Parent web is now a working app (builds clean): sign in, add child, generate
    enrollment QR, edit policy (screen time, bedtime, web filter, blocked apps,
    location), send "lock now"/"unlink". Verified with `npm run build`.
  - Backend: added `create-enrollment-token` Edge Function (parent-authed, mints
    single-use token, stores only the hash).
  - Android: full enrollment spine written — Gradle config, `SecureStore`
    (EncryptedSharedPreferences), `Api` (OkHttp → enroll/sync/heartbeat),
    `EnrollmentActivity` (disclosure → QR scan → enroll), `SyncWorker`,
    `PolicyStore`, `BootReceiver`, layouts/theme/icon.
    ⚠️ Not yet compiled here (no Android SDK in this environment) and FCM is
    stubbed (needs `google-services.json`). Build in Android Studio next.

## Where we are: v0.1 "foundation"

This commit is a **scaffold**: architecture, security, and compliance are designed and documented;
the backend data model and RLS are written; the Android app and parent web have real structure and
key files but are **not yet functional end-to-end**. Think of it as a well-laid foundation and frame,
not a finished house.

### Done in v0.1
- [x] Repo structure (monorepo: backend / child-app-android / parent-web)
- [x] Full docs: overview, architecture, security, compliance, features, data model
- [x] Backend: Postgres schema (`backend/supabase/schema.sql`)
- [x] Backend: RLS policies (`backend/supabase/policies.sql`)
- [x] Backend: Edge Function skeletons (enroll-device, device-sync, push-command, device-heartbeat)
- [x] Android: manifest, Device Admin receiver, foreground service + persistent notification, disclosure screen (skeletons)
- [x] Parent web: Vite/React skeleton with Supabase client + auth + dashboard shell
- [x] `.env.example` files, `.gitignore`

### NOT done (be honest with yourself)
- [ ] Nothing is wired end-to-end yet; no real enrollment, no real enforcement.
- [ ] No tests, no CI.
- [ ] Device Owner provisioning not implemented (only the Device Admin hook).
- [ ] No actual web filtering / location / lock implementations (designed, not coded).

## Phases

### Phase 1 — Make one loop work (MVP spine)
Goal: parent creates account → enrolls a device → sets ONE policy (block an app) → device enforces it.
- Supabase project live; run schema + policies; verify RLS with tests.
- Edge Functions deployed; enroll-device mints device JWT.
- Android: enrollment via QR, store JWT securely, pull policy, block one app, heartbeat.
- Web: auth, add child, show QR, simple policy toggle, live "last seen".

### Phase 2 — Core parental features
- Screen-time limits & schedules, per-app limits.
- Usage reports on dashboard.
- Lock/focus now (with emergency-call carve-out).
- Device Admin → Device Owner provisioning path + removal-resistance hardening.

### Phase 3 — Web filtering & location
- SafeSearch/DNS managed config; optional VPNService filter.
- Opt-in location sharing with retention limits.

### Phase 4 — Trust, polish, compliance pass
- Transparency screen in child app; friendly block/lock UX.
- Privacy policy, Data Safety form, store listing copy review against compliance doc.
- Accessibility, localization (Bengali + English first).
- Security review / pen-test.

### Phase 5 — Scale
- Load testing; Edge Function rate limits & quotas; DB indexing/partitioning for usage data.
- Observability (metrics, alerting on heartbeat-loss anomalies).
- Cost modeling; regionalization; abuse monitoring at scale.
- Incident response & data-retention automation.

## Honest expectations
A compliant, secure, large-scale parental-control platform is a **multi-month, multi-person effort**,
especially the Android DPC work and the Play review process. This foundation makes that work
organized and safe to build on — it does not compress it into one night. The most valuable thing here
is that the hard, easy-to-get-wrong parts (compliance boundaries, auth/trust model, data model) are
decided correctly up front.
