# 04 — Features

Legend: ✅ in v0.1 scaffold · 🔜 planned · 🚫 never (policy/ethics)

## Parent dashboard (website)
- 🔜 Sign up / sign in (Supabase Auth)
- 🔜 Create family, add child profiles
- 🔜 Enroll device via QR
- 🔜 Policy editor per child:
  - Daily screen-time limit + schedules (bedtime, school hours)
  - App list: allow / block / per-app time limit
  - Web filtering level (SafeSearch, category blocking)
  - Location sharing on/off
  - "Lock now" / "Focus now" temporary lockdown
- 🔜 Live device status (last seen, foreground app, battery)
- 🔜 Usage reports (per app, per day/week)
- 🔜 Location map (if enabled)
- 🔜 Unlink / revoke device

## Child Android app (single app)
- ✅ (scaffold) Foreground service + **mandatory persistent notification**
- ✅ (scaffold) Disclosure/consent screen
- ✅ (scaffold) Device Admin receiver (removal resistance hook)
- 🔜 Enrollment flow (scan QR → bind to child → store device JWT)
- 🔜 PolicyEngine: enforce app blocking & time limits
- 🔜 Web filtering (VPNService-based local filter or managed config + SafeSearch)
- 🔜 Usage stats collection (PACKAGE_USAGE_STATS) + heartbeat
- 🔜 Location reporting (only when enabled)
- 🔜 Lock screen / focus mode with **emergency-call access always available**
- 🔜 Sync worker (WorkManager) + FCM wake receiver
- 🔜 Device Owner provisioning path (QR/zero-touch) + Device Admin fallback

## Safety / ethics features (not optional)
- ✅ Child always sees the app and its notification
- 🔜 Child-visible "what my guardian can see" screen inside the app (transparency)
- 🔜 Emergency calls never blocked, even in lockdown
- 🔜 Age-appropriate tone; no shaming language

## Explicitly never (would be stalkerware / banned / unethical)
- 🚫 Hidden/stealth mode or hiding the icon
- 🚫 Reading SMS/chat/email **content**
- 🚫 Call recording or ambient microphone capture
- 🚫 Remote camera capture
- 🚫 Keylogging
- 🚫 Screenshotting other apps' content
- 🚫 Marketing to monitor spouses/partners/employees/adults
- 🚫 Promising the app is impossible to remove

## Feature design notes

### Screen time & app limits
Enforced on-device by `PolicyEngine`. When the limit is hit, the app shows a friendly full-screen
block ("Time's up for TikTok today") over the target app. Uses usage-stats + foreground-app
detection. Emergency dialer and the FamilyGuard app itself are never blocked.

### Web filtering
Two viable approaches, pick per platform constraints:
1. **Managed DNS / SafeSearch enforcement** via DPC managed configuration (lighter, less invasive).
2. **Local VPNService** that filters DNS/HTTP against category lists (more control, more battery, must
   disclose the VPN clearly — it shows a key icon, which aligns with our transparency requirement).
Start with (1), add (2) if needed.

### Location
Off by default. When a parent turns it on, the child app shows a notice, then reports coarse location
on a schedule (not continuous tracking). Retention is short. This is "family location sharing", framed
exactly like that.

### Lockdown / focus
Parent can trigger "lock now" or scheduled focus. The device shows a lock screen but **always allows
emergency calls**. This is a care tool, not a punishment tool — copy reflects that.
