# HTL Content Guard PC — Privacy Policy

_Last updated: 2026-10-16_

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

## Licensing

The separate desktop application checks your licence (free trial / premium) with
a licensing server using an anonymous, per-install identifier. That check
involves no browsing data and no personal identifiers such as your name or
email. The browser extension itself performs no licensing calls.

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
