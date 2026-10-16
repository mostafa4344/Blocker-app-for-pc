# Ship checklist — Content Guard

What's DONE and verified in-repo vs. what needs your hardware/accounts.

## ✅ Done & tested (in this repo / CI)
- Website/app/keyword/allowlist blocking, schedules, daily limits, lock modes.
- Uninstall & settings protection with time-delay up to "unlimited" (always a
  safe exit). Clock-tamper resistant.
- Trial→Premium licensing via Supabase (`setup.sql` applied). Trial anchored to a
  machine fingerprint so wiping app data does not reset it.
- Premium gating (paywall) with a safe exit; activation codes; payment-webhook
  template (`supabase/functions/payment-webhook`).
- Browser supported/unsupported lists + "close unsupported browsers".
- Force SafeSearch (Google/YouTube/Bing/DuckDuckGo) via hosts.
- MV3 extension (DNR, block page, partial blocking of Shorts/Reels/feeds).
- bn/en UI, onboarding, dashboard. Installers + CI for Win/macOS/Linux.

## ⬜ Needs YOU (hardware/accounts/credentials) — cannot be done from the cloud box
1. **Run on a real Windows PC**: download the CI `Content Guard-windows`
   artifact and run `install.ps1` as Administrator; confirm service, hosts
   blocking, app-closing, and uninstall-protection flow. (Or link Remote Control
   so I can drive it.)
2. **Code signing**: Windows OV/EV cert + sign every .exe; macOS Developer ID +
   notarization. Needed to avoid SmartScreen/Gatekeeper warnings. Then submit to
   Microsoft/Defender for reputation + AV false-positive whitelisting.
3. **Extension identity for force-install**: publish the extension to the
   Chrome Web Store (~$5 one-time) / Edge Add-ons / Firefox AMO, OR self-host a
   signed build; then paste the id into Settings. Tooling + guide in
   `packaging/extension/`. Until then the "lock extension" switch stays inert.
4. **Payment provider**: pick one (Stripe / bKash / Nagad / card). Wire its
   webhook to the Edge Function (`admin_grant_premium`). Set `WEBHOOK_SECRET` +
   `SERVICE_ROLE_KEY` secrets.
5. **(Optional) Allow `supabase.co` egress** for this cloud session so live
   licensing can be verified from here.

## ⬜ Planned next (I can build; network-level needs real-machine testing)
- DNS family-filter layer (DoH to a family resolver + block DNS/DoH changes).
- WFP (Windows) / NetworkExtension (macOS) / nftables (Linux) for system-wide,
  bypass-proof enforcement (close VPNs, alt-DNS, unknown browsers).
- Store listings for Chrome Web Store / Firefox Add-ons (one-click extension).

## ✅ Hardening added since first build
- Second stable install-id anchor: besides the machine id, the id is mirrored
  to a hidden home-dir file (`~/.contentguard-id`, outside the app data dir).
  Resolution prefers an already-saved id, then the anchor, then the machine id,
  then a random id — so wiping app data *or* regenerating the machine id keeps
  the same trial. Override path with `CONTENTGUARD_ID_FILE`.
- **Managed-browser extension (force-install + lock):** the agent writes the
  OS enterprise browser policy so the extension installs itself, can't be
  removed by the user, survives browser reinstall, and is re-asserted every
  ~15 s. New `browserpolicy` module (Windows registry + Linux managed dirs),
  Settings switch, and status. Needs a stable extension identity to arm —
  see `packaging/extension/README.md` (step C below).
- **Enforcement switches are now tighten-only while protected:** with
  protection above Off (or anything locked), the user can no longer untick
  "require extension" / "block unsupported browsers" / "lock extension" /
  "force SafeSearch" to open a bypass.
