# Chrome Web Store — everything ready to upload

This file has **everything you paste** into the Chrome Web Store, plus the exact
click-by-click steps. The hard parts (packaging, manifest, icons, privacy
policy) are already done. You only upload and copy-paste.

> The ZIP you upload: **`dist/content-guard-extension.zip`** (in this repo).

---

## PART 1 — What to do (one time, ~15 minutes)

1. Open **https://chrome.google.com/webstore/devconsole** and sign in with your
   Google account.
2. Pay the **one-time US$5** registration fee (needs a card). This is forever —
   not per extension, not yearly.
3. Click **“Add new item”** → upload **`dist/content-guard-extension.zip`**
   (drag-and-drop the file).
4. Fill the listing fields by copy-pasting from **PART 2** below.
5. For the screenshots: open the HTL Content Guard app in your browser
   (`http://127.0.0.1:17345`), take 1–3 screenshots (1280×800), and upload them.
   (Tell me if you want me to generate polished promo images instead.)
6. **Visibility:** choose **Unlisted** (works for force-install, not shown in
   search — simplest and least review friction) or Public. Either is fine.
7. Click **Submit for review.**

That’s it. Google reviews it (usually **1–3 days**).

---

## PART 2 — Copy-paste these fields

**Item name**
```
HTL Content Guard PC
```

**Summary (short description, ≤132 chars)**
```
Companion for the HTL Content Guard blocker: block pages, keyword & path blocking, and hide Shorts, Reels and feeds.
```

**Category:** `Workflow & Planning`  (or `Productivity`)

**Language:** English

**Detailed description**
```
HTL Content Guard helps you stay away from distracting and harmful websites so you
can focus on what matters.

This extension is the browser companion for the HTL Content Guard desktop app. On
its own it does nothing — it works together with the desktop app running on the
same computer, and enforces the rules you set there:

• Block websites, pages and whole categories (adult, gambling, social, etc.).
• Keyword and path blocking, and allow-lists (block everything except a few sites).
• Hide the most distracting parts of sites you still need — YouTube Shorts and
  recommendations, Facebook feed and Reels, Instagram Reels, X and LinkedIn feeds.
• A friendly block page that reminds you of your goal.

Your privacy is protected: the extension talks only to the HTL Content Guard app on
your own computer (localhost). It does not send your browsing anywhere — there
is no tracking, no analytics and no ads.

Get the free HTL Content Guard desktop app first; this extension connects to it
automatically.
```

**Privacy policy URL** (required — already written and hosted in the repo)
```
https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/PRIVACY.md
```

### Privacy practices tab — answers

- **Single purpose:**
  ```
  Enforce the website/content blocking rules the user configured in the Content
  Guard desktop app, by blocking pages and hiding distracting sections.
  ```
- **Permission justifications** (paste each where asked):
  - `declarativeNetRequest` →
    ```
    Used to block pages and network requests according to the user's own block list.
    ```
  - `tabs` + host permission `<all_urls>` →
    ```
    Reads the active tab's URL to decide whether that page should be blocked and to
    show the block page. Applied only to sites the user chose to block.
    ```
  - `storage` →
    ```
    Stores the user's block rules locally in the browser.
    ```
  - `alarms` / `idle` →
    ```
    Re-checks the rules periodically and when the user returns to the computer.
    ```
- **Data usage checkboxes:** do **not** tick any “collects/sells/transfers”
  category. Then tick the three certification boxes:
  - I do not sell or transfer user data to third parties, outside of approved use cases
  - I do not use or transfer user data for purposes unrelated to my item's single purpose
  - I do not use or transfer user data to determine creditworthiness or for lending purposes

---

## PART 3 — The ID is already handled

The ID was assigned the moment you uploaded the ZIP:
**`amlkhggimheblaboilcbjdjomlceckdp`** (it never changes, even after approval).

I have already **baked this id into the app as the built-in default**, so there
is nothing to paste and nothing to wait for me on:

1. **Today:** you finish the listing (PART 2) and submit for review.
2. **In 1–3 days:** Google approves. The extension is now live in the store.
3. From then on, in the HTL Content Guard app, **Settings → Browser protection →
   tick “Lock the extension into my browsers”** is all it takes — every
   Chrome/Edge/Brave/Opera/Vivaldi on that PC then auto-installs the extension,
   locked so it can’t be removed. No id to type (the “Extension ID” box can stay
   empty; it only exists for advanced overrides).

Edge Add-ons and Firefox AMO are optional extras later (same ZIP, free
accounts); the Chrome id already covers Chrome, Brave, Vivaldi and Opera.

---

## PART 4 — Privacy tab (copy-paste, field by field)

Open the **Privacy** tab (left menu of the item). Fill each box exactly:

**Single purpose** (one description box):
```
Enforce the website and content blocking rules the user set in the HTL Content Guard desktop app, by blocking pages and hiding distracting sections on the sites the user chose.
```

**Permission justifications** — a box appears for each permission. Paste:

- `declarativeNetRequest`:
```
Used to block pages and network requests according to the user's own block list.
```
- `storage`:
```
Stores the user's block rules locally in the browser.
```
- `tabs`:
```
Reads the active tab's URL to decide whether that page should be blocked and to show the block page. Only applied to sites the user chose to block.
```
- `alarms`:
```
Periodically re-checks the block rules.
```
- `idle`:
```
Re-applies the rules when the user returns to the computer.
```
- **Host permission** `<all_urls>` (box titled "Host permission justification"):
```
The user can choose to block any website, so the extension must be able to check and block pages on any domain they add to their block list.
```

**Remote code** — choose **"No, I am not using remote code."** (all our code is inside the package.)

**Data usage** — this is the important part, answer honestly:
- "Does this item collect or use data?" → the extension reads the active tab URL
  only to enforce blocking **on the user's own device**, and sends it **only to
  localhost (127.0.0.1)** — nothing is transmitted to us or any third party.
  - If it asks you to pick data types: you may tick **"Website content"**
    (because it reads the URL). Do NOT tick personal info, financial, health,
    authentication, location, etc.
  - For its purpose, choose **"App functionality"** only. Do NOT tick
    "Analytics", "Developer communications", "Advertising", or
    "personalization".
- Then tick the **three certification checkboxes** at the bottom:
  - I do not sell or transfer user data to third parties, outside of the approved use cases
  - I do not use or transfer user data for purposes that are unrelated to my item's single purpose
  - I do not use or transfer user data to determine creditworthiness or for lending purposes

**Privacy policy URL** (required):
```
https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/PRIVACY.md
```

Click **Save draft**.

---

## PART 5 — Distribution tab

- **Visibility:** **Public** (so anyone can install) — or **Unlisted** if you
  prefer it not to show in search. Either works with the lock-extension feature.
- **Distribution / regions:** **All regions.**
- **Pricing:** **Free.**

Click **Save draft.**

---

## PART 6 — Submit

Top-right: **Submit for review.** If a box turns red, it tells you which field is
missing — fill it and submit again. Review usually takes **1–3 days**.

That is the whole process. After approval nothing else is needed from the store
side; the lock-extension switch in the app already knows this extension's id.

---

## PART 7 — Which browsers does this cover?

The Chrome Web Store build covers every Chromium browser — **Chrome, Edge,
Brave, Opera, Vivaldi** (~70–75% of users). Firefox uses a different store
(**Firefox AMO**, free) — the same code works there; we can publish it later.

Even where the extension is not installed, the desktop app still enforces:
- **Website blocking (hosts file)** — works in *every* browser, system-wide.
- **Force SafeSearch** — every browser.
- **App blocking** — every app.
- **"Close unsupported browsers"** — shuts any browser that can't run the
  extension, so it can't be used to bypass.

So Firefox and others are not an open hole; they still get website + SafeSearch
blocking, and can be closed during focus. The extension only adds the extra
in-page features (keyword, Shorts/Reels/feed hiding) on top.
