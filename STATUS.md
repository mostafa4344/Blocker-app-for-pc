# Project status — honest completion

Updated: 2026-10-16. This is a candid snapshot, not marketing. "Done" means the
code is written and (where possible here) verified; it does **not** mean tested on
real devices at scale.

## Overall: a working core, not a finished product

| Area | State | % | Notes |
|---|---|---|---|
| Architecture & docs | ✅ Done | 95% | overview, architecture, security, compliance, data model, setup |
| Backend schema + RLS | ✅ Written | 90% | not yet run on a live DB (you run `setup_all.sql`) |
| Edge Functions | ✅ Written | 80% | not deployed/tested (needs your CLI); rate-limiting TODO |
| Parent web | ✅ Builds | 75% | auth, children, enroll QR, policy editor, usage, location, commands |
| Android child app | ✅ Written | 60% | compiles target; **not built/tested on a device here** (no SDK) |
| Push (FCM) | 🟡 Stubbed | 20% | code ready; needs your `google-services.json` |
| Web content filtering | 🔲 Designed | 10% | VPNService/DNS approach documented, not coded |
| Scale / load / abuse | 🔲 Designed | 5% | roadmap phase 5 |
| Security review / pen-test | 🔲 Not started | 0% | required before public launch |
| Play Store submission | 🔲 Your step | 0% | listing, Data Safety, review |

## What works end-to-end on paper (after you deploy)
Parent signs up → adds child → gets enrollment QR → child app scans → enrolls →
pulls policy → blocks an app / enforces bedtime & daily limit → reports usage +
status → parent sees it live → parent can lock/unlink. Location is opt-in.

## The ONLY things that cannot be done from this environment (they need you)
1. **Run the database setup** — `backend/supabase/setup_all.sql` in the Supabase
   SQL editor (needs your project; 1 paste). Optional: `retention.sql`.
2. **Deploy the Edge Functions** — `supabase functions deploy …` with the CLI
   (needs your login / project link).
3. **Build & device-test the Android app** — in Android Studio on a real phone
   (this cloud box has no Android SDK and no device).
4. **Firebase** for push — add `google-services.json`, uncomment `FcmService` + its
   manifest entry and Gradle deps.
5. **Publish to Play Store** — developer account, signing, Data Safety form,
   privacy-policy URL, review.

Exact steps for 1–2 are in `docs/09-SETUP-THIS-PROJECT.md` (filled in for your project).

## Known gaps / next engineering (not blockers to trying it)
- Edge Functions need rate limiting + input validation (zod) before real traffic.
- Web filtering (SafeSearch/DNS or VPNService) not implemented.
- Device Owner provisioning (QR/zero-touch) not implemented — only Device Admin.
- No automated tests yet beyond the web build check in CI.
- Per-app time limits: data model supports them; enforcement is block-only so far.

See `docs/05-ROADMAP.md` for the phased plan.
