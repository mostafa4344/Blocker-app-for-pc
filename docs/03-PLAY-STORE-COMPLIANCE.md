# 03 — Google Play Compliance (READ THIS BEFORE BUILDING FEATURES)

This is the document that keeps the app on the Play Store. Compliance is a **design constraint, not a
checkbox at the end.** If a feature request conflicts with this doc, this doc wins.

> Policy wording changes. Always verify against the live pages before a release:
> - Google Play Developer Program Policy (Device & Network Abuse / "Stalkerware")
> - Google Play **Families** policy: https://support.google.com/googleplay/android-developer/answer/9893335
> - Families preview (2026): https://support.google.com/googleplay/android-developer/answer/17122218
> - Target API level requirement: all apps must meet the latest target API level by **31 Aug 2026**.

## The core rule (stalkerware policy)

Google **permanently bans** apps that covertly surveil people. Monitoring apps are allowed **only**
when they are **exclusively designed and marketed for parental (family) monitoring or enterprise
management**, AND they:

1. **Do NOT present themselves as a spying or secret surveillance solution.**
2. **Do NOT hide or cloak tracking behavior**, or mislead users about it.
3. **Present a persistent notification AND a unique icon** that makes it clear the device is being
   monitored, whenever data is being collected/transmitted.

Violating any of these = removal + developer account termination risk.

### What this means for us — hard requirements

| # | Requirement | How we meet it |
|---|---|---|
| C1 | Persistent notification on child device whenever monitoring is active | A foreground service shows a non-dismissible notification: "FamilyGuard is active. Your parent/guardian can see activity on this device." |
| C2 | Visible, unique app icon — never hidden | App icon stays in launcher. No "hide icon" feature. Ever. |
| C3 | No "spy/secret/stealth" framing anywhere | Store listing, UI, docs, marketing all say "parental control" / "family". The word "spy" never appears as a feature. |
| C4 | Clear disclosure + consent at setup on the child device | `DisclosureActivity` lists exactly what is shared, shown before enrollment completes. Child taps "I understand". |
| C5 | Parental-only use enforced in onboarding | Setup states this is for a parent/guardian managing a minor's device. ToS prohibits other use. |
| C6 | No prohibited data types | We do NOT capture message content, call audio, keystrokes, screenshots of other apps, ambient mic/camera. |

## Families policy (because the user base is children)

If the app targets children (it does, on the child side), additional rules apply:

- **Data disclosure:** declare in the Play Console Data Safety form exactly what is collected from
  children, including via any SDKs. Keep it minimal.
- **No transmitting restricted identifiers** (e.g. advertising ID / AAID) from children or
  users of unknown age. Don't integrate ad SDKs on the child app. Keep the child app ad-free.
- **No long-term storage of age-signal data.**
- **COPPA / GDPR-K alignment:** verifiable parental consent, data minimization, deletion on request.
  The parent account is the consent anchor.

### Two "audiences", two listings
- **Child app** → targets children/teens → full Families compliance, ad-free, minimal data, clear
  disclosure.
- **Parent dashboard** → is a **website**, not a Play app, so Play listing rules apply only to the
  child app. (If a parent Android app is added later, it targets adults and has its own listing.)

## Permissions discipline
Request only what a feature needs, justify each high-risk permission, and prefer the modern managed
APIs over broad ones:

- Prefer **Device Policy Controller (DPC)** APIs (Device Owner/Profile Owner) over `AccessibilityService`
  for control. Google scrutinizes `AccessibilityService` heavily; using it for control can get apps
  rejected unless clearly justified. Use it only as a fallback and disclose it.
- `PACKAGE_USAGE_STATS` (usage reports) — justify in listing.
- `ACCESS_FINE_LOCATION` / background location — only if location feature is on; strong justification
  + runtime consent + the persistent notification.
- `QUERY_ALL_PACKAGES` — only if genuinely needed for app management; declare justification.

## Uninstall protection — the honest truth
- Device Owner mode makes the app the device administrator; Android then forbids a normal uninstall
  and the user can't simply disable it. This is the legitimate, Family-Link-style mechanism.
- **But:** Device Owner can only be provisioned on a **new or factory-reset device** (QR/NFC/zero-touch
  at setup). You cannot silently convert an already-set-up phone into Device Owner.
- **And:** it can still be removed by **factory reset from recovery** or **reflashing firmware**. No
  parental app is truly undeletable. We document this and design the dashboard to **detect** removal
  (heartbeat loss) rather than falsely promise prevention.
- On Android 16+, provisioning requires stronger trust signals than a bare QR — verify current
  Android Enterprise provisioning requirements per release.

## Marketing & copy rules (binding)
- ✅ "Parental controls", "family safety", "screen time", "guardian".
- ❌ "Spy", "secret", "hidden", "undetectable", "catch them", "monitor your spouse/employee".
- Every surface (store, web, in-app) must make it obvious the device is managed and by whom.

## Pre-submission checklist
- [ ] Persistent notification present and non-dismissible while active (C1).
- [ ] Icon always visible; no hide-icon code path exists (C2).
- [ ] No "spy/stealth/secret" language anywhere (C3).
- [ ] Disclosure+consent screen ships and is shown pre-enrollment (C4).
- [ ] Data Safety form matches actual collection; child app ad-free; no AAID transmit (Families).
- [ ] Each dangerous permission justified; AccessibilityService avoided or justified.
- [ ] Target API level current.
- [ ] Privacy policy published and linked.
- [ ] No prohibited capture (messages/calls/keystrokes/ambient).
