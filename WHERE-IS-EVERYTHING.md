# Where everything is (so you never lose it)

Everything is stored permanently on **GitHub**, independent of any Claude/cloud
session. Even if you lose this chat, the connection, or the cloud session, all
the code and files are safe here and you can copy them any time.

- **Repository:** https://github.com/mostafa4344/Blocker-app-for-pc
- **Branch with all the work:** `ccr-00759c63-vxxesd`

## How to get a file any time
1. Open the repository link above on github.com (sign in as mostafa4344).
2. Top-left, click the branch dropdown and choose **`ccr-00759c63-vxxesd`**.
3. Browse to the file and click it to read it.
4. To copy the text: click the **Raw** button (top-right of the file), then
   select-all and copy. The **Raw** link is also a permanent direct URL.
5. To download the whole project: green **Code** button → **Download ZIP**.

## Key files and their direct "Raw" links
(Replace nothing — these open the exact text to copy.)

**Buy / payment page (English):**
`web/purchase.html`
https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/web/purchase.html

**Install guide page (English, with the SmartScreen "Run anyway" steps):**
`web/install.html`
https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/web/install.html

**Supabase SQL for the purchase table:**
`supabase/purchase-requests.sql`
https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/supabase/purchase-requests.sql

**Supabase SQL for licensing (run once):**
`supabase/setup.sql`
https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/supabase/setup.sql

**Order-fulfilment guide (how to grant Premium after payment):**
`web/PURCHASE-SETUP.md`

**Privacy policy (used by the Chrome Web Store listing):**
`PRIVACY.md`
https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/PRIVACY.md

**Chrome Web Store listing text + steps:**
`packaging/extension/STORE-LISTING.md`

**Browser extension (source + the store ZIP you upload):**
`extension/` and `dist/content-guard-extension.zip`

**Windows ONE-CLICK installer (the easy one — double-click):**
`dist/HTL-Content-Guard-PC-Setup.exe`
https://raw.githubusercontent.com/mostafa4344/Blocker-app-for-pc/ccr-00759c63-vxxesd/dist/HTL-Content-Guard-PC-Setup.exe

**Windows app (zip) + PowerShell installer (advanced):**
`dist/ContentGuard-windows.zip`, `packaging/windows/install-online.ps1`, `packaging/windows/installer.nsi`

**Store images (icon, screenshots, promo):**
`packaging/extension/store-assets/`

**Project notes / decisions / status:**
`NOTES.md`, `docs/SHIP-CHECKLIST.md`, `CLAUDE.md`

## Important facts to keep
- **Chrome extension ID:** `amlkhggimheblaboilcbjdjomlceckdp` (never changes).
- **Supabase project URL:** `https://qhplhhrmathbfyejkzbr.supabase.co`
- **Supabase publishable key** (safe for the public page) is inside `web/purchase.html`.
- The **service-role key** (secret, for granting Premium) lives only in your
  Supabase dashboard — never put it in any public page or in GitHub.
