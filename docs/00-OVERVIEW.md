# 00 — Product Overview

## The idea in one paragraph

A parent creates a **family account** on a website. They add a **child** and enroll the child's
Android phone by installing **one app** and scanning a QR code. From then on, the parent signs in to
the website (from any device) and can: set screen-time limits, block/allow apps, enforce safe web
browsing, see where the phone is, and view usage reports. The child's app enforces these rules and
resists casual removal. The child always sees that the app is installed and active.

## The three components

| Component | Who uses it | Tech | Lives in |
|---|---|---|---|
| **Parent web dashboard** | Parents | React + Vite + Supabase JS | `parent-web/` |
| **Backend** | (services) | Supabase: Postgres, Auth, Realtime, Edge Functions | `backend/` |
| **Child Android app** | Installed on child's phone | Kotlin, Device Owner/Admin APIs | `child-app-android/` |

There is intentionally **no parent mobile app in v1** — the parent uses the website. (A parent app
can come later; see roadmap.) This keeps v1 scope sane.

## Core user journeys

### 1. Parent signs up
Parent → website → email/password (Supabase Auth) → creates `family` → adds a `child` profile.

### 2. Enroll a child device
Website shows an **enrollment QR code** (short-lived token). On the child phone, the single app is
installed and provisioned as **Device Owner** (ideally at new-device/factory-reset setup) or as a
**Device Admin + Accessibility** (fallback for already-set-up phones). The app shows the
**disclosure/consent screen**, then binds to the child profile using the token.

### 3. Parent sets rules
Website → pick child → set policy (allowed apps, daily limits, bedtime, web filter level, etc.).
Policy is saved in Postgres; a realtime event + push wakes the child app to pull the new policy.

### 4. Child app enforces
The child app continuously enforces the active policy locally (so it keeps working offline) and
reports usage/heartbeats back to the backend.

### 5. Parent monitors
Website shows last-seen, current foreground app, screen-time used, location (if enabled), and
reports. Everything the parent can see is something the child is told, up front, is visible.

## What "control" means here (scoped to Play-compliant features)

Allowed, transparent features we build toward:
- ⏱️ Screen time limits & schedules (bedtime, school hours)
- 🧩 App management: allow / block / time-limit specific apps
- 🌐 Safe browsing / web content filtering (SafeSearch, category blocking)
- 📍 Location sharing (with on-device notice)
- 📊 Usage reports (which apps, how long)
- 🔒 Device lock / "focus now" (temporary lockdown with emergency-call access)
- 🛡️ Removal resistance (Device Owner), with honest limits

Explicitly **out of scope** (would make this stalkerware):
- Reading private message contents, call recording, ambient mic/camera capture, hidden operation,
  keystroke logging. See `docs/03-PLAY-STORE-COMPLIANCE.md`.

## Scale ambition

The owner's goal is very large scale (hundreds of millions of users). The architecture is chosen
with that in mind (stateless Edge Functions, Postgres with RLS, realtime fan-out, device-side
enforcement so the backend is not in the hot path). But **v0.1 is a foundation, not a scaled
system.** Load testing, sharding, regionalization, and cost work come much later (roadmap phase 5).
