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
3. **Payment provider**: pick one (Stripe / bKash / Nagad / card). Wire its
   webhook to the Edge Function (`admin_grant_premium`). Set `WEBHOOK_SECRET` +
   `SERVICE_ROLE_KEY` secrets.
4. **(Optional) Allow `supabase.co` egress** for this cloud session so live
   licensing can be verified from here.

## ⬜ Planned next (I can build; network-level needs real-machine testing)
- DNS family-filter layer (DoH to a family resolver + block DNS/DoH changes).
- WFP (Windows) / NetworkExtension (macOS) / nftables (Linux) for system-wide,
  bypass-proof enforcement (close VPNs, alt-DNS, unknown browsers).
- Store listings for Chrome Web Store / Firefox Add-ons (one-click extension).
- Second stable location for the install id (registry/hidden file) to further
  resist trial reset.
