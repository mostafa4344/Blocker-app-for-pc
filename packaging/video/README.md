# Tutorial video (install & setup guide)

Everything needed to re-make the ~7-minute tutorial when the app changes.
Output: 1080p30 H.264, English neural voice-over (edge-tts, "Andrew"),
English + Bangla subtitles, a Bangla-burned copy, thumbnail, YouTube text.

All screens are real: a clean **demo copy** of the app (own data folder, fake
hosts file, port 17346, local license mock on :8788) driven by puppeteer-core
in a fresh headless Chrome. The real installed app is never touched.
SmartScreen / UAC can't be screenshotted, so `dialogs.mjs` draws them.
The installer pages come from `nsis/shot.nsi`: the real installer pages, but it
runs without admin and only unpacks into %TEMP% (`nsis/shoot.ps1` photographs it).

Steps (paths inside the scripts point at %TEMP%\cgvideo — copy this folder there):
1. `npm i puppeteer-core@23.11.1`, `pip install edge-tts pillow numpy`, ffmpeg on PATH.
2. `cargo build -p contentguard-agent` (debug build: honours the dev license key).
3. `reset.ps1` → `node capture.mjs` → `approve.ps1` (wait ~15 s, or POST
   /api/license/refresh) → `node capture2.mjs` → `node capture3.mjs` → `node dialogs.mjs` → `node logo.mjs`.
4. Installer pages: build `nsis/shot.nsi` with makensis, run `nsis/shoot.ps1`.
5. `python tts.py` (voice + word timings), `python render.py video`
   (or chunks: `video 0 140`, ... then concat), `python thumb.py`.
6. Mux audio + subtitles and burn the Bangla copy (see the ffmpeg lines in NOTES.md).

Edit the narration / Bangla lines / highlights in `script.py` only.
