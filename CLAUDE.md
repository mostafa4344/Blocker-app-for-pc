# CLAUDE.md — Content Guard (for future AI/dev sessions)

PC content blocker (Windows/macOS/Linux): website + app + keyword blocking,
schedules, daily limits, lock modes, uninstall/settings protection (time-delay),
force SafeSearch, browser supported/unsupported control, and a trial→premium
licensing model backed by Supabase.

## Workspace
- `crates/core` — pure logic, no OS calls, fully unit-tested (matcher, schedule,
  engine, lock, protection, challenge, clock, presets, store, model).
- `crates/agent` — the background service: `hosts` + `processes` enforcement,
  `safesearch`, local `api` (127.0.0.1) serving the `ui/`, `license` (+ `sync`
  loop), `fingerprint` (machine-derived install id), Windows `service`.
- `ui/` — vanilla ES-module web UI (bn/en), embedded in the agent via include_str!.
- `extension/` — MV3 browser extension (DNR rules, block page, partial blocking).
- `backend/` — Supabase-compatible license MOCK for local testing only.
- `supabase/` — `setup.sql` (run in Supabase), migrations, payment-webhook fn.
- `packaging/` — per-OS install/uninstall; `.github/workflows/ci.yml` builds all.

## Golden rules
- The agent is the single authority: every change is validated server-side in
  `app.rs`/`lock.rs`. Never trust the UI/extension for enforcement.
- Locks only tighten; lowering protection/uninstall go through `protection::Request`
  and ALWAYS keep a legitimate exit (cooldown/timeout). Never remove that exit.
- NO MITM HTTPS proxy, NO kernel rootkit tricks, NO disabling other security
  software — these get flagged as malware. Use OS-sanctioned mechanisms
  (hosts, DNS, WFP/NetworkExtension, browser extension).
- Keep it clippy-clean (`-D warnings`) on linux + `x86_64-pc-windows-gnu` +
  `x86_64-apple-darwin` (use `--no-default-features` for cross-check).

## Checks before committing
```
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
node --test extension/test/*.test.js ui/test/*.test.mjs
cd backend && npm test
```
Dev run: `CONTENTGUARD_DATA_DIR=./.dev/data CONTENTGUARD_HOSTS_FILE=./.dev/hosts \
CONTENTGUARD_PORT=17345 cargo run -p contentguard-agent -- run`

## Env overrides (dev/test)
`CONTENTGUARD_DATA_DIR`, `CONTENTGUARD_HOSTS_FILE`, `CONTENTGUARD_PORT`,
`CONTENTGUARD_SYNC_SECS`.

## Status: see NOTES.md (decisions) and docs/SHIP-CHECKLIST.md (what's left).
