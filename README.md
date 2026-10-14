# FamilyGuard — Transparent Parental Control Platform

> **Codename:** `FamilyGuard` (placeholder brand — rename before launch, check trademarks).
> **Status:** v0.1 foundation / scaffold. **Not production-ready.** See [`docs/05-ROADMAP.md`](docs/05-ROADMAP.md).

A parental-control platform where a **parent** signs in on a **website** and can see and manage
their **child's** Android phone (screen time, app blocking, safe web browsing, location sharing,
usage reports). The child's phone runs **one** app, which protects itself from casual removal using
Android's Device Owner / Device Admin APIs.

This project is built to be **compliant with Google Play policy**. That is not optional — it is the
single most important design constraint. See [`docs/03-PLAY-STORE-COMPLIANCE.md`](docs/03-PLAY-STORE-COMPLIANCE.md).

---

## ⚠️ Read this first — what this is and is NOT

This is a **transparent, consent-based parental control tool** for a parent to manage the device of
**their own minor child**. It is designed so the child always knows it is installed and running.

It is **NOT** and must never become:

- ❌ Covert/hidden "spy" software (stalkerware). Google Play permanently bans this.
- ❌ A tool to monitor a spouse, partner, employee, or any adult without their knowledge/consent.
- ❌ A secret audio/screen/message recorder with a hidden icon or "stealth mode".

**Hard rules baked into the design:**
1. The child device **always** shows a persistent notification + a visible app icon while active.
2. A clear **disclosure + consent screen** is shown on the child device during setup.
3. The app is **never** marketed or described as "spying" or "secret surveillance".
4. We **never** promise parents the app is impossible to remove. (Factory reset / recovery / reflash can remove it — see compliance doc.)

Breaking any of these turns a legal product into banned stalkerware. Do not do it.

---

## Repository layout

```
.
├── README.md                     # this file
├── docs/                         # architecture, security, compliance, roadmap
├── backend/                      # Supabase: SQL schema, RLS policies, Edge Functions
│   └── supabase/
├── child-app-android/            # The single Android app installed on the child's phone (Kotlin)
└── parent-web/                   # The website parents log into (React + Vite + Supabase JS)
```

## Backend note

Per the project owner's instruction, all backend files live under [`backend/`](backend/) for now.
They are written for **Supabase** (Postgres + Auth + Realtime + Edge Functions) so they can be
transferred into your own Supabase project later. See [`backend/README.md`](backend/README.md).

## Where to start reading

1. [`docs/00-OVERVIEW.md`](docs/00-OVERVIEW.md) — the product in one page
2. [`docs/01-ARCHITECTURE.md`](docs/01-ARCHITECTURE.md) — how the pieces fit
3. [`docs/03-PLAY-STORE-COMPLIANCE.md`](docs/03-PLAY-STORE-COMPLIANCE.md) — the rules that keep you on the store
4. [`docs/05-ROADMAP.md`](docs/05-ROADMAP.md) — what is done, what is next
