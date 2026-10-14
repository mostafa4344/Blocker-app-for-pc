# 08 — Google Play Data Safety mapping

The Play Console **Data safety** form must match what the app actually does.
This maps our data to the form so the declaration is accurate. Re-verify before
each release; an inaccurate form is a policy violation on its own.

## Data types collected (child app)

| Play category | Item | Collected? | Shared? | Purpose | Notes |
|---|---|---|---|---|---|
| Location | Approximate location | Only if parent enables | No | App functionality (family location sharing) | Off by default; disclosed on device |
| Location | Precise location | Optional | No | App functionality | Prefer coarse; justify if precise |
| App activity | App interactions / usage | Yes | No | App functionality (screen-time, limits, reports) | Aggregate foreground time only |
| App info & performance | Diagnostics | Optional | No | Crash/diagnostics | If a crash SDK is added, declare it |
| Device or other IDs | Device ID | Yes | No | App functionality (identify the enrolled device) | Internal id we generate; NOT advertising ID |

## Data types explicitly NOT collected
- Messages (SMS/email/IM) content — **not collected**
- Photos/videos, voice/audio — **not collected**
- Contacts, calendar — **not collected**
- Web browsing history content — **not collected** (web filtering acts on
  categories/DNS locally; we don't log the child's browsing history to the server
  in v1 — if that changes, update this form)
- Advertising ID — **not collected / not transmitted**

## Security & handling answers
- **Is data encrypted in transit?** Yes (TLS).
- **Can users request deletion?** Yes (parent can delete child/device/account).
- **Is data processed ephemerally?** Partly (FCM wake carries no data); stored
  data follows the retention policy.
- **Committed to Play Families policy?** Yes.

## Account (parent) side
The parent account (email/auth) is collected for the website. If you later ship a
**parent Android app**, its own Data Safety form covers email/auth; that app
targets adults, separate from the child app's Families declaration.

## Permissions justification (for the listing / review)
| Permission | Why | Form/justification |
|---|---|---|
| PACKAGE_USAGE_STATS | Screen-time + app limits + reports | Core parental-control feature; user grants Usage Access |
| SYSTEM_ALERT_WINDOW | Show the "time's up" block screen | Needed to display block over a limited app |
| ACCESS_FINE/BACKGROUND_LOCATION | Family location sharing | Only when parent enables; persistent notification shown |
| FOREGROUND_SERVICE (special use) | Mandatory persistent status notification | Compliance + continuous enforcement |
| RECEIVE_BOOT_COMPLETED | Restart status/enforcement after reboot | Keep protection active |

## Pre-release check
- [ ] Form matches code (grep the app for any SDK that collects more).
- [ ] No ad/analytics SDK transmitting child identifiers.
- [ ] Privacy policy URL live and linked.
- [ ] Each permission above still used; remove any that aren't.
