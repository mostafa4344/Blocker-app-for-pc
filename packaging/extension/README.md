# Locking the extension into browsers (force-install)

This is how the "Lock the extension into my browsers" switch in **Settings →
Browser protection** actually works, and the one step that needs **your**
accounts to turn it on.

## What the app already does

Once an extension **identity** is configured, the agent writes an *enterprise
browser-management policy* (the official Google/Microsoft/Mozilla mechanism)
that **force-installs** the extension:

- It installs itself when the browser starts — the user clicks nothing.
- Its **Remove** button is locked; it can't be deleted or disabled from
  `chrome://extensions` / `about:addons`.
- The policy lives in the machine registry / managed-policy dir, **not** in the
  browser profile, so it **survives the browser being reinstalled**.
- The agent **re-asserts it every ~15 s**, so deleting the policy key (which
  needs admin) is undone almost immediately.
- Browsers where the extension can't run, or that haven't checked in, are
  closed by the existing "require extension" / "block unsupported browsers"
  switches — so there's no un-covered browser to escape to.

Where the policy is written:
- **Windows:** `HKLM\SOFTWARE\Policies\{Google\Chrome, Microsoft\Edge,
  BraveSoftware\Brave, Chromium, Vivaldi}\ExtensionInstallForcelist`, and
  `…\Mozilla\Firefox\ExtensionSettings` for Firefox.
- **Linux:** `/etc/opt/chrome/policies/managed/contentguard.json` (and the
  Edge/Brave/Chromium/Vivaldi equivalents), plus Firefox `policies.json`.

## The one thing that needs you: a stable identity

Force-install **requires** the extension to have a fixed id and an update URL.
Pick one path.

### Path A — publish to the stores (simplest, most robust)

1. **Chrome Web Store** (covers Chrome, Brave, Vivaldi, Opera): one-time ~$5
   developer account, upload `extension/`, publish (unlisted is fine).
2. **Microsoft Edge Add-ons**: free developer account, upload the same zip.
3. **Firefox AMO** (addons.mozilla.org): free; upload and get a **signed** XPI.

Then in **Settings → Browser protection** paste the Chrome Web Store id into
**Extension ID** and leave **Update URL** empty (it defaults to the Web Store).
Turn the switch on. Done.

### Path B — self-host (no stores)

1. Generate a signing key and keep it **secret** (never commit it):
   ```
   openssl genrsa 2048 | openssl pkcs8 -topk8 -nocrypt -out key.pem
   ```
2. Get the extension id this key produces:
   ```
   node packaging/extension/derive-chromium-id.mjs key.pem
   ```
3. Pack a signed `.crx` for the `extension/` folder with that key (e.g. Chrome's
   "Pack extension", or a CRX3 packer). Note the `version` from
   `extension/manifest.json`.
4. Fill in `update.xml.template` → `update.xml` with the id, the absolute
   `https` URL of the `.crx`, and the version. Host `update.xml` and the `.crx`
   (the repo's `raw.githubusercontent.com` URLs work, like the Windows build).
5. In Settings paste the id into **Extension ID** and the `update.xml` URL into
   **Update URL**. Turn the switch on.

For Firefox self-hosting you still need a **signed** XPI (AMO signs XPIs for
free even if unlisted); set its URL as the Firefox install URL.

## Why this isn't flagged as malware

It uses only each vendor's documented enterprise policy API — the same keys an
IT department sets. No MITM proxy, no injected DLLs, no tampering with the
browser binary. See the top-level `CLAUDE.md` golden rules.
