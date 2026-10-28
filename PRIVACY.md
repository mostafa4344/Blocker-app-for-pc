# HTL Content Guard PC — Privacy Policy

_Last updated: 2026-10-25_

HTL Content Guard is a self-control / website-and-app blocker. It is designed so
that your browsing data **never leaves your computer**.

## What the browser extension does

The HTL Content Guard browser extension works only together with the HTL Content Guard
desktop application running on the same computer. It:

- Reads the address of the page in the active tab to decide whether that page
  should be blocked, and to hide distracting sections (such as Shorts, Reels and
  feeds) according to the rules you chose in the desktop app.
- Talks **only** to the HTL Content Guard desktop app on your own computer, over the
  local loopback address `127.0.0.1` (localhost). It does not send your browsing
  to us or to any third party.

## Data we collect

**None.** We do not operate any server that receives your browsing history,
page contents, keystrokes, or personal information from the extension. The
extension has no analytics and no advertising.

- **Browsing activity** (the URL of the active tab) is used **locally** only, to
  enforce the blocks you configured, and is not transmitted off the device.
- The extension stores a small amount of local configuration (your block rules,
  synced from the desktop app) in the browser's local storage on your device.

## What the desktop app sends to our server

The desktop application contacts our server a few times a day. It sends only:

- an **anonymous device identifier** (a random-looking code derived from the
  computer, not your name, email or Windows account) — used for your free trial
  and Premium licence;
- the **app version**;
- a small **anonymous health report**, so we can find and fix problems: the
  Windows version (e.g. "Windows 10 Pro"), whether website blocking works,
  your protection level, how many lists are always on, in how many browsers
  the extension is connected, the result of the last automatic update, and —
  if the app crashed — the technical crash message (no personal data).

It **never** sends the sites you visit, the apps you use, your block lists,
keystrokes, screenshots, your name, email or files.

The app also downloads ready-made block lists and, when available, **automatic
updates**. Updates are digitally signed by us; the app refuses any file whose
signature doesn't match.

## Payments

If you buy Premium by card or PayPal, the payment is processed by our reseller
Paddle (paddle.com), which acts as merchant of record and handles your payment
details under its own privacy policy. We receive only your device identifier
and which plan you bought, so Premium can be turned on for your computer.

The browser extension itself performs no licensing or payment calls.

## Permissions and why they are needed

- **declarativeNetRequest** — to block pages and requests according to your rules.
- **tabs** / host access (`<all_urls>`) — to read the active tab's URL so the
  right rule can be applied on any site you choose to block, and to show the
  block page.
- **storage** — to keep your block rules locally in the browser.
- **alarms** / **idle** — to re-check the rules periodically and when you return
  to the computer.

## Contact

Questions about this policy: open an issue on the project's repository.
