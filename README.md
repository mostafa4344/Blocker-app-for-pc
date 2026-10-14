# FocusFort: Blocker App for PC

Windows / macOS / Linux-এর জন্য একটা website, app আর social media blocker। এতে আছে time limit, schedule, lock mode, আর partner-ভিত্তিক uninstall protection।

- 📊 Research (৫৭টি অ্যাপ): [docs/research/competitor-feature-research.md](docs/research/competitor-feature-research.md)
- 📝 সিদ্ধান্ত, অবস্থা ও পরের কাজ: [NOTES.md](NOTES.md)

## গঠন

| ফোল্ডার | কী আছে |
|---|---|
| `crates/core` | Platform-নিরপেক্ষ নিয়ম: block list, schedule, দৈনিক সীমা, lock, protection request, trusted clock |
| `crates/agent` | Background service: hosts file + app blocking, local API ও UI (`127.0.0.1:17345`), sync, Windows service |
| `ui/` | বাংলা/English web UI (agent-এর ভেতরে embed করা) |
| `extension/` | Chrome/Edge/Brave/Firefox extension (MV3): block page, path/keyword/allowlist, Shorts/Reels লুকানো |
| `backend/` | অস্থায়ী partner/sync server (Node, কোনো dependency নেই, JSON file) |
| `supabase/` | Supabase-এ সরানোর জন্য SQL schema ও নির্দেশনা |
| `packaging/` | Windows/macOS/Linux install ও uninstall script |

## Development-এ চালানো

```sh
# hosts file আর data আলাদা জায়গায় রেখে, admin ছাড়াই
FOCUSFORT_DATA_DIR=./.dev/data FOCUSFORT_HOSTS_FILE=./.dev/hosts cargo run -p focusfort-agent -- run
# তারপর browser-এ: http://127.0.0.1:17345/

# Partner backend (ঐচ্ছিক)
cd backend && npm start          # http://localhost:8787 → Settings-এ sync server হিসেবে দিন
```

Extension লোড করা: `chrome://extensions` → Developer mode → Load unpacked → `extension/` ফোল্ডার।

## Test

```sh
cargo test && cargo clippy --all-targets -- -D warnings
node --test extension/test/*.test.js
cd backend && npm test
```

## Install (আসল ব্যবহার)

CI প্রতিটি OS-এর জন্য `FocusFort-<os>` নামে একটা artifact বানায়:

- **Windows:** Administrator PowerShell-এ `install.ps1`
- **macOS / Linux:** `sudo ./install.sh`

Uninstall করতে হলে, uninstall protection চালু থাকলে আগে অ্যাপের **সুরক্ষা → Uninstall করার অনুরোধ** দিতে হবে, approval পেলে তারপর uninstaller চালাতে হবে।

## Agent commands

`focusfort-agent run | service | uninstall-check | cleanup | status | version`
