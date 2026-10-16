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
5. For the screenshots: open the Content Guard app in your browser
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
Content Guard
```

**Summary (short description, ≤132 chars)**
```
Companion for the Content Guard blocker: block pages, keyword & path blocking, and hide Shorts, Reels and feeds.
```

**Category:** `Workflow & Planning`  (or `Productivity`)

**Language:** English

**Detailed description**
```
Content Guard helps you stay away from distracting and harmful websites so you
can focus on what matters.

This extension is the browser companion for the Content Guard desktop app. On
its own it does nothing — it works together with the desktop app running on the
same computer, and enforces the rules you set there:

• Block websites, pages and whole categories (adult, gambling, social, etc.).
• Keyword and path blocking, and allow-lists (block everything except a few sites).
• Hide the most distracting parts of sites you still need — YouTube Shorts and
  recommendations, Facebook feed and Reels, Instagram Reels, X and LinkedIn feeds.
• A friendly block page that reminds you of your goal.

Your privacy is protected: the extension talks only to the Content Guard app on
your own computer (localhost). It does not send your browsing anywhere — there
is no tracking, no analytics and no ads.

Get the free Content Guard desktop app first; this extension connects to it
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

## PART 3 — The ID, and when I do the rest (your timing question)

**Important: the app already fully works without the ID.** The ID only switches
on the *force-install + lock* feature. So there is no “waiting on me”:

1. **Today:** you upload + pay. (Nothing more from me is required — it’s all ready.)
2. **In 1–3 days:** Google approves and shows you the item’s **ID** — a 32-letter
   string on the item’s dashboard page (and in its store URL:
   `chrome.google.com/webstore/detail/<name>/THIS-IS-THE-ID`).
3. **The moment you have the ID**, you do **one** thing — no waiting for me:
   - Open the Content Guard app → **Settings → Browser protection**
   - Paste the ID into **“Extension ID”**, leave **Update URL** empty
   - Tick **“Lock the extension into my browsers”** → Save
   - From then on, every Chrome/Edge/Brave on that PC auto-installs the extension,
     locked so it can’t be removed.

**So the answer to “when do I give you the ID and when do you work?”** — you
don’t have to hand it to me and wait. It’s a value you type into the app
yourself, and it works instantly. If you’d *also* like me to bake the ID in as
the built-in default (so new installs get it with nothing to type), just send me
the ID in chat any time and I’ll make a one-line change and push it — a couple of
minutes, whenever it arrives. No deadline, no blocking.

Edge Add-ons and Firefox AMO are optional extras later (same ZIP, free
accounts); Chrome’s ID already covers Chrome, Brave, Vivaldi and Opera.
