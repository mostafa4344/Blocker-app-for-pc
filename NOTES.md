# NOTES: সিদ্ধান্ত, অবস্থা ও পরের কাজ

> আপনি বলেছিলেন প্রশ্ন থাকলে নিজে সিদ্ধান্ত নিয়ে নোট রাখতে। এখানে সব সিদ্ধান্ত, কী পরীক্ষা হয়েছে, কী বাকি আছে, আর কোথায় আপনার সিদ্ধান্ত লাগবে, সব লেখা আছে।

## ১. আমার নেওয়া সিদ্ধান্ত

| # | বিষয় | সিদ্ধান্ত | কারণ |
|---|---|---|---|
| 1 | অ্যাপের নাম | **ContentGuard** (অস্থায়ী) | Research-এর "Fortress" ধারণা থেকে নেওয়া। পরে সহজে বদলানো যাবে: `ContentGuard` লেখা খুঁজে replace করলেই হবে। |
| 2 | ভাষা | Core/agent **Rust**-এ; UI হলো সাধারণ HTML/JS | Rust দ্রুত, নিরাপদ, আর এক code দিয়ে Windows/macOS/Linux তিনটাতেই চলে। |
| 3 | Desktop UI | Agent নিজেই `http://127.0.0.1:17345`-এ UI দেখায় (browser-এ খোলে) | এই container-এ Tauri-র জন্য দরকারি WebKit নেই। একই UI পরে Tauri দিয়ে আলাদা window + tray icon হিসেবে মোড়ানো যাবে, কোনো বড় পরিবর্তন লাগবে না। **পরের ধাপ।** |
| 4 | Website block | **hosts file** (agent) + **browser extension** (DNR rules) | hosts file সব browser-এ কাজ করে। extension দিয়ে path (`youtube.com/shorts`), keyword, allowlist আর partial block হয়। |
| 5 | App block | চলমান process-এর নাম মিলিয়ে বন্ধ করা | Cold Turkey/FocusMe-ও এভাবে করে। explorer.exe, svchost-এর মতো system process কখনো বন্ধ করা হয় না। |
| 6 | Uninstall protection | ৪টি স্তর: বন্ধ / সচেতন / প্রতিশ্রুতিবদ্ধ (২৪–৭২ ঘণ্টা অপেক্ষা) / পার্টনার লক | Research doc-এর প্রস্তাব অনুযায়ী। **প্রতিটি স্তরেই বৈধ বের হওয়ার পথ আছে** (cooldown বা partner timeout)। কেউ চিরতরে আটকে থাকে না, তাই antivirus অ্যাপটাকে malware হিসেবে ধরবে না। |
| 7 | Uninstall-এর পদ্ধতি | Installer একটা স্বাভাবিক Windows service বসায়। Uninstaller চালালে আগে `contentguard-agent uninstall-check` চলে, approval না থাকলে uninstall থেমে যায়। | পুরোপুরি স্বচ্ছ পদ্ধতি। কোনো লুকানো কৌশল নেই। |
| 8 | Fortress স্তর | Code-এ রাখা হয়নি, শুধু গাইড হিসেবে প্রস্তাব | User-এর account Standard করে দেওয়া, আর admin password partner-এর কাছে রাখা, এটা OS-এর নিজস্ব নিয়ম। পরের version-এ একটা "setup wizard" বানানো যায়। |
| 9 | ঘড়ি বদলে ফাঁকি | `TrustedClock`: wall clock আর monotonic time না মিললে monotonic time বিশ্বাস করা হয় | ঘড়ি সামনে বা পেছনে নিয়ে session শেষ করা যায় না। |
| 10 | Lock-এর নিয়ম | Lock থাকা অবস্থায় শুধু "আরও কড়া" করা যায়: সাইট যোগ করা যাবে, সরানো যাবে না; সীমা কমানো যাবে, বাড়ানো যাবে না | Cold Turkey-র মডেল। Agent server-side যাচাই করে, তাই UI বাইপাস করলেও কাজ হয় না। |
| 11 | Practice mode | Onboarding-এর পর প্রথম ৭ দিন lock সর্বোচ্চ ২ ঘণ্টা | নতুন user যেন ভুল করে নিজেকে আটকে না ফেলেন (Cold Turkey-র সবচেয়ে বড় অভিযোগ)। Settings থেকে বন্ধ করা যায়। |
| 12 | Backend | এখন **Node (কোনো dependency নেই) + JSON file** (`backend/`) | আপনি বলেছিলেন আপাতত কোথাও রাখতে। Supabase-এর জন্য SQL schema (`supabase/`) আর data export script তৈরি আছে। |
| 13 | Backend কোথায় লাগে | শুধু Partner approval আর tamper alert-এ | বাকি সব offline কাজ করে। Privacy-first: কোনো browsing history বা screenshot server-এ যায় না। |
| 14 | Email | `outbox` table-এ জমা থাকে, console-এ log হয় | এখনো কোনো email service যুক্ত করা হয়নি। Supabase-এ নেওয়ার সময় Resend/SMTP যোগ করতে হবে। |
| 15 | ডিফল্ট ভাষা | বাংলা, সাথে English | — |
| 16 | Local API-র নিরাপত্তা | শুধু 127.0.0.1; Host header যাচাই; পরিবর্তন করতে `X-ContentGuard` header লাগে; শুধু extension origin-কে CORS অনুমতি | অন্য কোনো website যেন agent-এ কিছু বদলাতে না পারে। |

## ২. যা পরীক্ষা করা হয়েছে (এই Linux container-এ)

- ✅ Rust unit tests: ৩৬টি (rules, schedule, limit, lock, protection, clock, hosts file)
- ✅ Extension rule tests: ৪টি; backend API tests: ৪টি
- ✅ `cargo clippy -D warnings` (Linux) + Windows/macOS-এর জন্য type-check
- ✅ Agent চালিয়ে API পরীক্ষা: session চালু → hosts file-এ ১০৮ লাইন; lock থাকা অবস্থায় সাইট সরাতে গেলে আটকে যায়; header ছাড়া request ও ভুল Host বাতিল হয়; random text টাইপ করে session থামানো যায়
- ✅ Playwright দিয়ে UI: onboarding → preset → protection → session; সব পেজে কোনো JS error নেই
- ✅ আসল Chromium-এ extension: `example.com`, `youtube.com/shorts/…`, keyword URL সব block page-এ চলে যায়
- ✅ Partner flow পুরোটা: request → partner-কে email → approve link → agent approval পায় → `uninstall-check` একবার অনুমতি দেয় → partner-কে "uninstalled" email যায়

## ৩. যা এখনো পরীক্ষা হয়নি ⚠️

- **Windows-এ আসল চালানো** (service, hosts file, app বন্ধ করা)। Code compile-check হয়েছে, কিন্তু চালানো হয়নি। CI-তে Windows build আছে। প্রথমবার একটা Windows PC-তে `packaging/windows/install.ps1` চালিয়ে দেখতে হবে।
- macOS-এ চালানো।
- Supabase SQL আসল Postgres-এ চালানো হয়নি (container-এ Postgres root হিসেবে চালানো গেল না)।
- Firefox-এ extension চালানো (manifest-এ Firefox সাপোর্ট আছে)।
- Partial block-এর CSS selector (YouTube/Facebook-এর HTML প্রায়ই বদলায়, তাই নিয়মিত দেখতে হবে)।

## ৪. জানা সীমাবদ্ধতা

- hosts file wildcard বোঝে না, তাই common subdomain (www, m, mobile…) আলাদা করে লেখা হয়। বাকিগুলো extension ধরে। পরের ধাপ: Windows Filtering Platform / macOS Network Extension দিয়ে system-wide DNS filter, তখন extension ছাড়াও সব কাজ করবে।
- Allowlist mode আর path/keyword block শুধু extension থাকলে কাজ করে। তাই "extension ছাড়া browser বন্ধ" option রাখা হয়েছে।
- Incognito-তে extension চালাতে user-কে browser settings-এ অনুমতি দিতে হয়। Store-এ প্রকাশের পর enterprise policy দিয়ে এটা বাধ্যতামূলক করা যাবে।
- App-এর দৈনিক সীমা গোনা হয় app চালু থাকার সময় ধরে, app সামনে আছে কিনা দেখে নয়। Foreground tracking পরের version-এ।
- Agent বন্ধ থাকা অবস্থায় ঘড়ি সামনে নেওয়া হলে সেটা ধরা যায় না। ভবিষ্যতে সমাধান: server time বা NTP দিয়ে যাচাই।
- কেউ Administrator হলে OS reinstall করে সবকিছু সরাতে পারবেন। এটা সব blocker-এর সীমাবদ্ধতা; research doc-এর অধ্যায় ৫ দেখুন।

## ৫. আপনার সিদ্ধান্ত লাগবে

1. **নাম ও brand:** ContentGuard রাখবেন, নাকি অন্য নাম?
2. **Code signing certificate** (Windows EV/OV, Apple Developer ID): release-এর আগে কিনতে হবে, না হলে SmartScreen warning দেখাবে।
3. **Supabase project:** তৈরি হলে URL আর key দিন, আমি Edge Function port করে দেব।
4. **Email service:** Resend নাকি SMTP?
5. **দাম/মডেল:** Free + Pro (partner, sync) নাকি এককালীন দাম?
6. **Chrome Web Store / Firefox Add-ons**-এ extension প্রকাশের account।

## ৬. পরের ধাপের পরিকল্পনা

1. Tauri shell: tray icon, আলাদা window, Windows-এ MSI installer (WiX)।
2. Windows-এ আসল পরীক্ষা ও bug fix।
3. System-wide DNS filter (WFP)।
4. Fortress setup wizard (Standard account গাইড)।
5. Foreground app tracking, Pomodoro, calendar।
6. Mobile companion app (partner-এর জন্য approve করার সুবিধা)।

---

# আপডেট — ১৫ অক্টোবর ২০২৬ (দ্বিতীয় ধাপ)

আপনার নতুন নির্দেশ অনুযায়ী বড় পরিবর্তন করা হয়েছে।

## নতুন সিদ্ধান্ত ও কাজ

| বিষয় | যা করা হয়েছে |
|---|---|
| **নাম** | পুরো প্রজেক্ট **Content Guard**-এ rename করা হয়েছে (crate, binary, service, env var `CONTENTGUARD_*`, API header `X-ContentGuard`, UI সব)। |
| **Email/partner** | বাদ দেওয়া হয়েছে। Resend/email দরকার নেই। পুরনো Node email backend সরানো হয়েছে। Partner protection level আর UI থেকে সরানো হয়েছে; এখন সুরক্ষা = Off / সচেতন / প্রতিশ্রুতিবদ্ধ (time-delay)। |
| **Time delay unlimited** | Committed (time-delay) এখন ১ দিন থেকে ৩০/৯০/৩৬৫ দিন, এমনকি **"আনলিমিটেড"** পর্যন্ত বেছে নেওয়া যায়। (আনলিমিটেড = কারিগরিভাবে ~১০০ বছর, নিচের ⚠️ দ্রষ্টব্য দেখুন।) |
| **Supabase** | আপনার দেওয়া URL ও publishable key বিল্ট-ইন ডিফল্ট হিসেবে বসানো হয়েছে (`crates/agent/src/license.rs`)। publishable key client-এ রাখা নিরাপদ। `supabase/migrations/` -এ লাইসেন্সিং schema ও RPC (register_device, get_license, redeem_code) আছে — আসল Postgres-এ চালিয়ে যাচাই করা হয়েছে। |
| **Freemium (trial → premium)** | ৪ দিনের ফ্রি ট্রায়াল, তারপর Premium লাগবে। ট্রায়াল server-side (install_id দিয়ে) ট্র্যাক হয় যাতে ডেটা মুছে reset করা কঠিন হয়। ট্রায়াল শেষ হলে নতুন ব্লক যোগ/জোরদার করতে Premium লাগে (402 paywall), কিন্তু **আগের ব্লক চলতেই থাকে** আর ব্লক কমানো সবসময় যায়। অফলাইনে ১৪ দিন grace, তারপর Premium আবার যাচাই করতে হয়। কোড দিয়ে Premium চালু হয় (`redeem_code`); পেমেন্ট webhook পরে যুক্ত হবে। |
| **Browser supported/unsupported** | `SUPPORTED_BROWSERS` (Chrome, Edge, Brave, Opera, Vivaldi, Yandex, Firefox, LibreWolf… — extension চলে, keyword ও adult স্ক্যান হয়) ও `UNSUPPORTED_BROWSERS` (Safari, Pale Moon, qutebrowser, Falkon, DuckDuckGo browser… — স্ক্যান অসম্ভব) তালিকা। Settings-এ দেখা যায়। **"unsupported ব্রাউজার বন্ধ করে দিন"** option চালু করলে ব্লক চলাকালীন ওই ব্রাউজারগুলো বন্ধ হয়ে যায়। |
| **লোকাল mock backend** | `backend/` এখন Supabase-compatible লাইসেন্স mock (একই RPC path)। এটা দিয়ে ট্রায়াল→Premium পুরো প্রবাহ container-এ টেস্ট করা হয়েছে। |

## যা টেস্ট হয়েছে (এই container-এ)
- Rust: ৪০টি (core ৩১ + agent ৯, licensing সহ)। Extension ৪, backend mock ৩। সব pass।
- clippy (Linux + Windows + macOS cross-check) পরিষ্কার।
- Supabase SQL আসল Postgres-এ চালিয়ে register→trial→redeem→premium→expired যাচাই।
- Agent + mock backend end-to-end: ট্রায়ালে ব্লক যোগ হয়, ট্রায়াল শেষে add → **402 paywall**, ব্লক কমানো তখনও কাজ করে, কোড দিয়ে Premium → আবার সব খোলে, ব্যবহৃত কোড আবার চলে না।
- UI (Playwright): Premium, Settings (browser তালিকা সহ), Protection (time-delay সহ) — কোনো JS error নেই।

## ⚠️ গুরুত্বপূর্ণ সতর্কতা ও আপনার সিদ্ধান্ত
1. **"আমি আপনাকে PC-তে connect করেছি" — এই session থেকে আপনার আসল PC-তে পৌঁছানো যায়নি।** এটা একটা Linux ক্লাউড container; এখানে কোনো remote device যুক্ত দেখা যায়নি। তাই আমি Windows PC-তে install করতে পারিনি — শুধু container-এ build ও trial করেছি। আপনার PC-তে চালাতে: CI থেকে `Content Guard-windows` artifact নামিয়ে `install.ps1` চালান (Administrator)। অথবা Remote Control যুক্ত করলে আমি সরাসরি চালাতে পারব (settings → Remote Control)।
2. **"আনলিমিটেড time delay" ও "Premium কিনতে বাধ্য করা" — আইনি/নৈতিক ঝুঁকি।** কাউকে একদম বের হতে না দেওয়া, বা টাকা না দিলে নিজের PC-র নিয়ন্ত্রণ ফিরে না পাওয়া — এটা antivirus/Windows "malware/PUA" হিসেবে ধরতে পারে এবং app store/পেমেন্ট প্রসেসর বাতিল করতে পারে। তাই আমি রেখেছি: আনলিমিটেড হলেও কারিগরি সীমা ~১০০ বছর (চিরস্থায়ী bug এড়াতে), আর Premium শেষ হলেও ব্লক *কমানো* ও uninstall-request সবসময় চালু থাকে। ১০ কোটি ব্যবহারকারীর লক্ষ্য হলে এই ভারসাম্য জরুরি। আপনি চাইলে আরও কড়া করা যায়, কিন্তু ঝুঁকি বাড়বে — সিদ্ধান্ত আপনার।
3. **Trial-reset দুর্বলতা:** এখন `install_id` শুধু `license.json`-এ থাকে; ফাইল মুছলে নতুন ট্রায়াল পাওয়া যেতে পারে। শক্ত করতে `install_id` একটা আলাদা স্থিতিশীল জায়গায় (registry/hidden file) রাখতে হবে — পরের ধাপ।
4. **Supabase migration আপনাকে একবার apply করতে হবে:** `supabase db push` (অথবা SQL editor-এ `migrations/20261015000000_licensing.sql` + `seed.sql`)। নইলে লাইসেন্স সার্ভার কাজ করবে না (app তখন শুধু লোকাল ট্রায়ালে চলবে)।
5. **পেমেন্ট এখনো যুক্ত হয়নি।** এখন `redeem_code` দিয়ে Premium (টেস্ট কোড: PREMIUM-LIFETIME, PREMIUM-30D, PREMIUM-365D)। bKash/Nagad/কার্ড যোগ করতে একটা পেমেন্ট webhook দরকার যেটা কেনার পর `codes` টেবিলে কোড বসাবে বা সরাসরি `accounts.plan='premium'` করবে। কোন পেমেন্ট মাধ্যম চান, জানালে যোগ করব।
6. **publishable key টি repo-তে আছে** (ডিফল্ট হিসেবে)। এটা publishable বলেই শেয়ারযোগ্য, কিন্তু আসল নিরাপত্তা নির্ভর করে RLS + RPC-র উপর (সেভাবেই করা হয়েছে: anon key শুধু RPC চালাতে পারে, টেবিল সরাসরি পড়তে/লিখতে পারে না)।

---

# আপডেট — ১৫ অক্টোবর ২০২৬ (তৃতীয় ধাপ)

- **Supabase এক-ফাইল setup:** `supabase/setup.sql` — Supabase SQL editor-এ paste করে Run করলেই সব তৈরি (টেবিল + RLS + RPC + payment webhook-এর জন্য admin RPC + টেস্ট কোড)। আসল Postgres-এ যাচাই: idempotent; register→trial, redeem→premium, admin_grant_premium (service_role) কাজ করে; anon admin function বা টেবিল **সরাসরি পড়তে পারে না** (permission denied)।
- **Trial-reset শক্ত করা হয়েছে:** `install_id` এখন **মেশিন-ভিত্তিক** (Windows: MachineGuid, macOS: IOPlatformUUID, Linux: /etc/machine-id — salt দিয়ে hash)। অ্যাপের ডেটা ফোল্ডার পুরো মুছে ফেললেও একই `install_id` থাকে, তাই ডেটা মুছে নতুন ট্রায়াল পাওয়া যায় না (container-এ যাচাই করা হয়েছে)। শুধু OS reinstall করলে বদলায় — সেটাই সীমা।
- **Premium দেওয়া (webhook/ম্যানুয়াল):** service-role দিয়ে `select admin_grant_premium('<device_id>', 30);` (null = আজীবন), অথবা `admin_create_code(...)` দিয়ে কোড বানিয়ে user-কে দিন। Device ID অ্যাপে **Settings → (advanced)** এ দেখা যায়।
- পরের ধাপ (আমি করব): পেমেন্ট webhook template, আর আরও কিছু robustness।

---

# আপডেট — ১৫ অক্টোবর ২০২৬ (চতুর্থ ধাপ): Filtering architecture + SafeSearch

**আর্কিটেকচার সিদ্ধান্ত (গবেষণার পর):** শুধু DNS নয় — স্তরে স্তরে (defense in depth):
1. DNS category filtering (adult-এর মূল ভিত্তি — পরের ধাপ)
2. Browser extension (keyword/path/SafeSearch — আছে)
3. hosts + app block (আছে)
4. পরে WFP/Network Extension দিয়ে system-wide
— **MITM HTTPS proxy করা হবে না** (Net Nanny CVE-র মতো ঝুঁকি, antivirus malware ধরে, app ভাঙে)।

**এই ধাপে করা হয়েছে — Force SafeSearch (hosts স্তরে, সব ব্রাউজারে):**
- Settings → ব্রাউজার সুরক্ষা → "SafeSearch জোর করে চালু রাখুন"।
- Google (forcesafesearch), YouTube (restrict), Bing (strict), DuckDuckGo (safe) — এদের "safe" endpoint-এর IP-তে hosts redirect। HTTPS ভাঙে না (safe endpoint আসল ডোমেইনের জন্য বৈধ cert দেয়), কোনো decryption নেই, malware-নিরাপদ।
- DNS fail করলে Google/YouTube-এর জন্য documented fallback IP (216.239.38.120) ব্যবহার হয়; প্রতি ৬ ঘণ্টায় re-resolve।
- কোনো সার্চ ইঞ্জিন পুরো block করা থাকলে SafeSearch pin বাদ যায় (block অগ্রাধিকার পায়)।
- container-এ DNS বন্ধ থাকলেও fallback দিয়ে ২৫টি hosts entry লেখা যাচাই করা হয়েছে; ১৫টি agent test পাস।

**পরের ধাপ (আমি করব):** DNS family-filter স্তর (agent থেকে family-safe DoH + DNS বদল আটকানো), তারপর WFP system-wide enforcement।

---

# আপডেট — ১৬ অক্টোবর ২০২৬ (পঞ্চম ধাপ)

- **সেটিংস ব্যাকআপ (export/import)** UI-তে যোগ — নতুন PC-তে সব লিস্ট/নিয়ম ফিরে পাওয়া যায়; import আগের মতোই lock ও Premium নিয়ম মানে।
- **DNS family-filter স্তর (ডিফল্ট বন্ধ, opt-in):** `dns.rs` — Cloudflare for Families (family) ও CleanBrowsing (strict) resolver; Windows `netsh` / macOS `networksetup` / Linux resolv.conf-এর জন্য pure command builder লেখা ও unit-test করা। Settings-এ option যোগ। **আসল system-DNS বদলানোর execution অংশটা এখনো wire করা হয়নি** — এটা OS-ভিত্তিক ও ঝুঁকিপূর্ণ, তাই আপনার Windows PC Remote Control-এ যুক্ত হলে ওখানে টেস্ট করে চালু করব (নইলে কারো ইন্টারনেট ভাঙতে পারে)। builder গুলো তৈরি ও যাচাই করা আছে।
- **Remote Control:** আপনার PC-তে Claude Desktop app বা `claude remote-control` চালালে আমি সরাসরি install/টেস্ট করতে পারব। এখনো কোনো ডিভাইস যুক্ত দেখা যায়নি।

---

# আপডেট — ১৬ অক্টোবর ২০২৬ (ষষ্ঠ ধাপ): অফিসে থাকাকালীন অটোনমাস কাজ

- নতুন category preset: **Gambling (জুয়া), Dating, AI-help (ChatGPT/Chegg ইত্যাদি)**।
- **পুরো pipeline Linux-এ আসল end-to-end টেস্ট করা হয়েছে** (Windows-এর বিকল্প হিসেবে): config → লক-করা session → hosts blocking (৯ লাইন) → SafeSearch pin (forcesafesearch IP) → license trial — সব একসাথে কাজ করেছে, UI-তে কোনো JS error নেই। স্ক্রিনশট chat-এ পাঠানো হয়েছে।

## "১০০%" এর সৎ হিসাব
**ক্লাউড থেকে যা সম্ভব — সব সম্পূর্ণ ও যাচাই করা (এই অংশে ১০০%):**
blocking engine, schedule, limit, lock (সব ধরন), time-delay protection (unlimited), licensing trial→premium (Supabase লাইভ), fingerprint anti-reset, paywall gating, SafeSearch, browser supported/unsupported + block, extension, bn/en UI, backup, installers, CI, ৫৩টি অটোমেটেড টেস্ট।

**বাকি ~১০% — শুধু আপনার মেশিন/অ্যাকাউন্ট লাগে (ক্লাউড থেকে অসম্ভব):**
1. আসল Windows PC-তে install + টেস্ট (Remote Control যুক্ত করলে আমি করব)।
2. DNS/WFP system-wide execution আসল মেশিনে টেস্ট করে চালু করা (builder তৈরি আছে)।
3. Code signing (Windows OV/EV + macOS notarization) — certificate কিনতে হবে।
4. পেমেন্ট gateway (bKash/Nagad/Stripe) — webhook template তৈরি, gateway অ্যাকাউন্ট দিলে লাগাব।

## PC যুক্ত হলে আমি যা করব (ক্রম)
1. CI artifact `Content Guard-windows` অথবা সরাসরি build → `install.ps1` (Administrator) দিয়ে install।
2. আসল Windows-এ যাচাই: service চালু, hosts blocking, app বন্ধ, SafeSearch, uninstall-protection, trial→premium।
3. DNS execution + পরে WFP system-wide — আসল মেশিনে টেস্ট করে ধাপে ধাপে চালু।

# আপডেট — ১৬ অক্টোবর ২০২৬ (সপ্তম ধাপ): আসল PC-তে চালু + anti-reset আরও শক্ত

- **অ্যাপ এখন আপনার আসল Windows PC-তে চালু** (one-line installer দিয়ে)। UI ইংরেজিতে,
  trial চালু, practice mode on — স্ক্রিনশটে নিশ্চিত।
- **Trial-reset প্রতিরোধ আরও শক্ত করা হলো:** install-id এখন দুটি জায়গায় anchor —
  (১) machine id, (২) home-dir-এ লুকানো ফাইল `~/.contentguard-id` (app data dir-এর
  বাইরে)। অগ্রাধিকার: license-এ সেভ করা id → anchor ফাইল → machine id → random।
  ফলে app data মুছলে *বা* machine id পাল্টালেও একই trial থাকে। test যোগ করা হয়েছে
  (মোট agent test ১৮→২০)। `CONTENTGUARD_ID_FILE` দিয়ে path override করা যায়।
- Browser extension এখনও "not connected" দেখাচ্ছে — এটা ব্যবহারকারীকে একবার
  `chrome://extensions` → Developer mode → Load unpacked দিয়ে লোড করতে হবে
  (installer `...\ContentGuard\extension` ফোল্ডার extract করে রাখে)। Chrome Web Store
  publish করলে এক-ক্লিকে হবে (আপনার dev account লাগবে)।

# আপডেট — ১৬ অক্টোবর ২০২৬ (অষ্টম ধাপ): extension জোর করে বসানো + bypass বন্ধ

ব্যবহারকারীর দাবি: অ্যাপ থেকে এক ক্লিকে extension বসবে, protection on থাকলে user
মুছতে পারবে না, browser/app reinstall-এও থেকে যাবে, না থাকলে ওই browser চলবে না।

যা করা হলো (সব কমিট + টেস্টেড, ৩ target-এ clippy-clean):
- **নতুন `browserpolicy` module:** browser-এর অফিসিয়াল **enterprise policy**
  দিয়ে extension **force-install** — নিজে install হয়, "Remove" লক, browser
  profile নয় বরং registry/managed-dir-এ থাকে (browser reinstall-এও টেকে), আর
  agent প্রতি ~১৫ সেকেন্ডে পুনরায় বসায় (policy key মুছলেও ফিরে আসে)।
  Windows: `HKLM\SOFTWARE\Policies\...\ExtensionInstallForcelist` (Chrome/Edge/
  Brave/Chromium/Vivaldi) + Firefox `ExtensionSettings`. Linux: managed JSON +
  policies.json। pure builder-গুলো unit-tested।
- **Bypass বন্ধ:** protection on (বা কিছু locked) থাকলে এখন
  require_browser_extension / block_unsupported_browsers / managed_extension /
  force_safe_search — এই switch-গুলো **বন্ধ করা যায় না** (tighten-only)। আগে
  user টিক তুলে দিয়ে ফাঁকি দিতে পারত; সেটা core lock.rs-এ বন্ধ করা হলো।
- UI (Settings → Browser protection): "Lock the extension into my browsers"
  switch + id/URL (advanced)। bn/en স্ট্রিং যোগ।

**যা শুধু আপনার লাগবে (এটা ছাড়া force-install arm হয় না):** extension-এর স্থায়ী
পরিচয় — (ক) Chrome Web Store/Edge/AMO-তে publish (~$৫ এককালীন), অথবা (খ) নিজে
host। পুরো গাইড + টুল: `packaging/extension/README.md`, `derive-chromium-id.mjs`,
`update.xml.template`। id বসালেই switch চালু হবে। নিরাপত্তার কারণে signing key
repo-তে রাখিনি (public হলে যে কেউ আমাদের ID দিয়ে ভুয়া extension বানাতে পারত)।

টেস্ট: core 31→32, agent 20→26। সব পাস।

# আপডেট — ১৮ অক্টোবর ২০২৬: রিব্র্যান্ড → "HTL Content Guard PC"

ব্যবহারকারীর সিদ্ধান্ত: অ্যাপের নাম "HTL Content Guard PC" (HTL = Halal Tech Life)।
শুধু **ব্যবহারকারী-দৃশ্যমান** নাম বদলানো হলো — UI title/tagline, settings footer,
extension manifest name + default_title + popup/blocked পেজ, PRIVACY.md,
STORE-LISTING.md, store screenshot + promo। ভেতরের টেকনিক্যাল আইডি **অপরিবর্তিত**
(crate নাম contentguard-*, বাইনারি, CONTENTGUARD_* env, data dir, Windows service,
Firefox gecko id extension@contentguard.app, রেজিস্ট্রি policy key, Chrome extension
id amlkhggimheblaboilcbjdjomlceckdp) — এগুলো বদলালে এখনকার ইনস্টল + extension id
ভাঙত। ZIP পুনরায় বানানো হয়েছে; ব্যবহারকারী Package ট্যাবে নতুন ZIP re-upload করবে
(একই store item, একই id)।

---

# আপডেট — ২৫ অক্টোবর ২০২৬ (রাতের কাজ): আপনার Windows PC-তে আসল বিল্ড + ফাঁক বন্ধ

এই session আপনার PC-তেই চলছে। Rust (GNU toolchain), WinLibs mingw-gcc, NSIS ইনস্টল করে
এখানেই বিল্ড/টেস্ট করা হয়েছে। Windows-এ সব Rust টেস্ট পাস।

## ⚠️ জরুরি: আপনার PC-তে এখন যা পাওয়া গেছে
- ContentGuard service ইনস্টল ও চলছে, protection = Committed (২৪ ঘণ্টা), **কিন্তু কোনো ব্লক
  সক্রিয় ছিল না** — লিস্ট ছিল, কিন্তু session/schedule ছাড়া আগের version কিছুই ব্লক করত না।
- ট্রায়াল শেষ (১৬ অক্টোবর শুরু) — server-ও "expired" বলছে, অর্থাৎ trial-reset প্রতিরোধ কাজ করছে।
- Browser extension যুক্ত নেই।

## নেওয়া সিদ্ধান্ত (আপনি সকালে দেখে নেবেন)
| # | সিদ্ধান্ত | কারণ |
|---|---|---|
| R1 | **"সবসময় ব্লক" (always-on) লিস্ট** যোগ। Adult ও Gambling preset ডিফল্টভাবে always-on। | ক্ষতিকর কন্টেন্ট "মাঝে মাঝে" ব্লক করার জিনিস নয়। আগে session না চালালে কিছুই ব্লক হতো না। |
| R2 | **Protection চালু থাকলে** always-on লিস্ট, চালু schedule ও চালু daily limit — সব "শুধু আরও কড়া" করা যায়। সাইট সরানো/বন্ধ/মুছে ফেলা যায় না। বদলাতে হলে আগে protection কমানোর অনুরোধ (cooldown) লাগে। | আগে protection থাকলেও লিস্ট থেকে সাইট মুছে ফেলা যেত — সবচেয়ে বড় ফাঁক। বৈধ বের হওয়ার পথ (cooldown) রাখা হয়েছে। |
| R3 | hosts ফাইল প্রতি ৬ সেকেন্ডে যাচাই করে আবার বসানো হয়। | আগে কেউ hosts ফাইল থেকে ব্লক অংশ মুছলে agent আর লিখত না। |
| R4 | License server ঠিকানা আর UI/config থেকে বদলানো যায় না (শুধু dev-এর env var `CONTENTGUARD_BACKEND_URL`/`_KEY`)। | নিজের ভুয়া server বসিয়ে বিনা পয়সায় Premium নেওয়া যেত। |
| R5 | Protection/লক চালু থাকলে Windows service **Stop মানে না**; অনুমোদিত uninstall-এর পর ১৫ মিনিট মানে। জোর করে kill করলে Windows ৩ সেকেন্ডে আবার চালু করে (recovery)। | `sc stop ContentGuard` দিয়ে এক লাইনে সব বন্ধ করা যেত। Qustodio/Net Nanny-র মতো parental-control অ্যাপও এভাবে করে; বৈধ পথ (uninstall request) আছে, তাই malware নয়। |
| R6 | Installer এখন **upgrade-safe**: চলমান exe rename করে নতুন বসায়, service রেখে দেয়, data রাখে। | নতুন R5-এর কারণে আগের "stop → delete → create" পদ্ধতি কাজ করত না। |
| R7 | নতুন সেটিং **"Private/Incognito window বন্ধ"** — Chrome/Edge/Brave/Vivaldi/Chromium/Firefox-এর অফিসিয়াল enterprise policy দিয়ে। Protection চালু থাকলে বন্ধ করা যায় না। | Incognito-তে extension চলে না, তাই keyword/path ব্লক এড়ানোর সবচেয়ে সহজ পথ ছিল। |
| R8 | Tor Browser (tor.exe) "unsupported browser" তালিকায়। | Tor নিজের DNS ব্যবহার করে, hosts-ব্লক মানে না। |
| R9 | Uninstall-এর সময় hosts অংশ + browser policy পরিষ্কার করা হয় (`cleanup`)। | আনইনস্টলের পর যেন কোনো অবশিষ্ট ব্লক/policy না থাকে (store/antivirus নিয়ম)। |
| R10 | Adult/জুয়া/ডেটিং-এর মতো সংবেদনশীল লিস্টের ভেতরের সাইটের নাম UI-তে **দেখানো হয় না** (শুধু সংখ্যা + নতুন যোগ করার ঘর)। | এই তালিকা নিজেই ট্রিগার হয়ে যেত। আন্তর্জাতিক মানের অ্যাপগুলোও এভাবে লুকিয়ে রাখে। |
| R11 | Onboarding-এ ডিফল্ট: Adult + জুয়া (সবসময় ব্লক) + Shorts; সুরক্ষা "সচেতন" (প্রস্তাবিত)। | বেশিরভাগ ইউজার ডিফল্টই রেখে দেন, তাই ডিফল্ট নিরাপদ হওয়া উচিত। |
| R12 | Extension ID UI থেকে সরানো; agent নিজে রাখে। DNS ফিল্টার অপশন UI থেকে লুকানো। | ভুল ID দিয়ে লক করা extension চুপচাপ বন্ধ করা যেত। DNS অপশন এখনো কাজ করে না, তাই দেখানো বিভ্রান্তিকর। |
| R13 | একই প্রস্তুত লিস্ট দুবার যোগ করলে ডুপ্লিকেট না বানিয়ে আগেরটার সাথে মেশে। | বাগ ছিল। |
| R14 | Version 1.1.0। | — |

## রাতে যা টেস্ট করে পাস হয়েছে (dev mode, এই PC-তে)
- ৪ দিনের ট্রায়াল শুরু; ট্রায়াল শেষে নতুন ব্লক/সেশনে Paywall, **কিন্তু আগের ব্লক চলতেই থাকে**।
- Premium কোড চালু হয়; ব্যবহৃত/ভুল কোডে পরিষ্কার বাংলা বার্তা।
- Always-on ব্লক সাথে সাথে hosts-এ বসে (৪৮৬ লাইন); hosts মুছলে ৫ সেকেন্ডে ফিরে আসে।
- API দিয়ে ৯ রকম ফাঁকির চেষ্টা — সব আটকেছে (`node .dev/bypass-test.mjs`)।
- আনইনস্টল: অনুরোধ ছাড়া/ভুল লেখায়/অপেক্ষার আগে আটকায়; অপেক্ষার পর একবারই অনুমতি দেয়।
- Rust টেস্ট ৫৯টি, extension ৪, backend ৩ — সব পাস; clippy পরিষ্কার। exe শুধু Windows-এর নিজস্ব DLL ব্যবহার করে।

## ⚠️ সকালে আপনাকে যা করতে হবে
1. **নতুন installer চালান:** `dist\HTL-Content-Guard-PC-Setup.exe` → UAC-তে "Yes"। (আমি "Yes" চাপতে পারি না, তাই আসল ইনস্টল/service-এর নতুন আচরণ — Stop না মানা, upgrade — এখনো আসল service-এ টেস্ট হয়নি।) আপনার আগের সেটিংস থাকবে।
2. **Supabase-এ একবার চালান:** `supabase/migrations/20261025000000_fixed_trial_length.sql` (SQL Editor-এ paste → Run)। এটা ছাড়া যে কেউ ৩৬৫ দিনের ফ্রি ট্রায়াল নিতে পারে।
3. ইনস্টলের পর অ্যাপে আপনার Adult লিস্ট খুলে "সবসময় ব্লক রাখুন" চালু করুন (আপনার ট্রায়াল শেষ, তাই আগে Premium লাগবে)।
4. এই ফোল্ডার git repo নয় — কিছু commit/push করা হয়নি।

## বাকি কাজ (পরের session)
- আসল installer দিয়ে ইনস্টল → service Stop প্রত্যাখ্যান, kill করলে আবার চালু, upgrade, uninstall পুরো প্রবাহ যাচাই।
- Schedules/limits পাতার শেষ UI পরীক্ষা; English লেখার পুরো পরিষ্কার; আরও ভাষা (আরবি/উর্দু/ইন্দোনেশিয়ান/তুর্কি)।
- Extension Chrome Web Store-এ প্রকাশ (force-install ও incognito সুরক্ষার পূর্ণ সুবিধার জন্য)।
- Dev টুল: `.dev/run.sh` (dev agent, port 17346), Rust GNU + `~/tools/winlibs` + `~/tools/nsis-3.11`।

---

# আপডেট — ২৫ অক্টোবর ২০২৬ (সকাল): আসল PC-তে ইনস্টল ও যাচাই + ৪টি নতুন ভাষা

## আসল PC-তে (Administrator হিসেবে) যা প্রমাণিত হয়েছে ✅
- নতুন installer protection চালু অবস্থাতেই upgrade করেছে; সেটিংস অক্ষত।
- `sc stop ContentGuard` (Admin) → ব্যর্থ (error 1052)। service `NOT_STOPPABLE`।
- Admin হিসেবে প্রসেস kill → ৭ সেকেন্ডের মধ্যে নতুন PID-এ আবার চালু।
- hosts ফাইল থেকে ব্লক অংশ মুছে ফেলা → ৯ সেকেন্ডে আবার ফিরে আসে (৩২৪ লাইন)।
- পুরনো config-এর migration: আপনার "Adult content" লিস্ট আপনাআপনি "সবসময় ব্লক" হয়েছে; pornhub.com-এ সংযোগ আটকায়।

## নতুন সিদ্ধান্ত
| # | সিদ্ধান্ত | কারণ |
|---|---|---|
| R15 | Config version 2 + একবারের migration: পুরনো Adult/জুয়া লিস্ট → always-on। | পুরনো ইউজাররা যা ব্লক করতে চেয়েছিলেন তা আসলে ব্লক হচ্ছিল না। |
| R16 | নতুন ভাষা: **আরবি ও উর্দু (ডান-থেকে-বাম layout সহ), ইন্দোনেশিয়ান, তুর্কি** — অ্যাপ, ব্লক পেজ ও extension popup-এ। মোট ৬টি ভাষা। | Halal Tech Life-এর মূল দর্শক (মধ্যপ্রাচ্য, পাকিস্তান/ভারত, ইন্দোনেশিয়া, তুরস্ক)। |
| R17 | ভাষাগুলো আলাদা ফাইলে (`ui/lang-xx.js`); `ui/test/i18n.test.mjs` নিশ্চিত করে সব ভাষায় একই key ও placeholder আছে (CI-তে চলে)। | নতুন লেখা যোগ করলে কোনো ভাষা যেন পিছিয়ে না পড়ে। |
| R18 | অব্যবহৃত ২০টি লেখা মুছে ফেলা হয়েছে। প্রস্তুত লিস্টের নাম ইউজারের ভাষায় তৈরি হয়। | অনুবাদের খরচ কমানো; আরবি ইউজার "Adult content" না দেখে নিজের ভাষায় নাম দেখবেন। |
| R19 | সুরক্ষিত schedule/limit-এ 🔒 চিহ্ন, "মুছুন" বাটন লুকানো। | আগে বাটন দেখাত কিন্তু চাপলে error আসত — বিভ্রান্তিকর। |
| R20 | ZIP ফাইলগুলো এখন সঠিক `/` path দিয়ে তৈরি। | Windows PowerShell `\` দিয়ে zip বানায়, যা Chrome Web Store প্রত্যাখ্যান করে। |
| R21 | Extension version 1.1.0 (নতুন ভাষা + "সবসময় ব্লক" কারণ ব্লক পেজে)। নতুন `dist/content-guard-extension.zip` Chrome Web Store-এ re-upload করতে হবে। | — |

## এখনো বাকি / আপনার করণীয়
1. **Supabase-এ SQL চালান:** `supabase/migrations/20261025000000_fixed_trial_length.sql` (৩৬৫ দিনের ট্রায়াল ফাঁক বন্ধ করতে)।
2. **Chrome Web Store-এ নতুন extension zip আপলোড** (`dist/content-guard-extension.zip`)।
3. নতুন ভাষার অনুবাদ একজন স্থানীয় ভাষাভাষীকে দিয়ে একবার দেখিয়ে নেওয়া ভালো (আমি সাবধানে লিখেছি, তবে store-এ প্রকাশের আগে native review আন্তর্জাতিক মান)।
4. Code signing certificate (SmartScreen সতর্কতা দূর করতে) — আগের মতোই বাকি।

---

# আপডেট — ২৫ অক্টোবর ২০২৬ (দুপুর): একাধিক লিস্ট, ব্রাউজার-পাহারা, admin-নিয়ন্ত্রিত লিস্ট

| # | সিদ্ধান্ত / কাজ | কারণ |
|---|---|---|
| R22 | হোমে "কী কী ব্লক করবেন" এখন **একাধিক বাছাই** (chip), "সব বাছুন/সব বাদ দিন"; প্রথমবার সোশ্যাল মিডিয়া + শর্ট ভিডিও প্রস্তাবিত। প্রতিটি লিস্টের জন্য একই সময় ও লকের আলাদা সেশন। | ইউজারের অনুরোধ: একটা বা একসাথে কয়েকটা। |
| R23 | এক্সটেনশন এখন নিজের ব্রাউজারের নাম পাঠায় (chrome/edge/brave/opera/yandex/generic/firefox)। প্রতিটি ব্রাউজার আলাদাভাবে যাচাই হয়। | আগে Chrome-এ এক্সটেনশন থাকলে এক্সটেনশন ছাড়া Edge-কেও "সুরক্ষিত" ধরা হতো — ফাঁক। পুরনো 1.0.0 এক্সটেনশন আপডেট না হওয়া পর্যন্ত আগের মতো চলবে। |
| R24 | "যে ব্রাউজারে ফিল্টার কাজ করছে না, সেটা বন্ধ" — **একটাই সুইচ**। এক্সটেনশন না বসলে/কাজ না করলে সেই ব্রাউজার নিজে থেকেই অসমর্থিত ধরা হয় এবং ব্লক চলাকালীন ৯০ সেকেন্ড পর বন্ধ হয়। সুইচ চালু করলে এক্সটেনশন force-install-ও চালু হয়। Settings-এ এখন চলমান প্রতিটি ব্রাউজারের অবস্থা দেখায়। | ইউজারের অনুরোধ। |
| R25 | প্রস্তুত লিস্ট এখন **Supabase টেবিল `preset_lists`** থেকে আসে (admin edit করেন, অ্যাপ ১৫ মিনিটে সিঙ্ক)। admin যা যোগ করেন আসে, যা সরান যায়, ইউজারের নিজের যোগ করা থেকে যায়। গাইড: `docs/ADMIN-PRESET-LISTS.md`। | ইউজারের অনুরোধ; নতুন version ছাড়াই লিস্ট আপডেট। |
| R26 | সার্ভার-ডেটার নিরাপত্তা: ভুল এন্ট্রি বাদ, `*` কখনো নয়, একবারে অর্ধেকের বেশি মুছলে উপেক্ষা, সংখ্যা/দৈর্ঘ্যের সীমা। সাইটের নামে শুধু অক্ষর/সংখ্যা/`-._` গ্রহণযোগ্য (hosts ফাইলে বাড়তি লাইন ঢোকানো ঠেকাতে) — ইউজারের টাইপ করা সাইটেও প্রযোজ্য। | সার্ভার হ্যাক হলেও বা admin ভুল করলেও কারো কম্পিউটার যেন ক্ষতিগ্রস্ত না হয়। |

আসল PC-তে ইনস্টল ও যাচাই: v1.1.0, service `NOT_STOPPABLE`, Chrome/Edge শনাক্ত, Adult ব্লক চালু।
টেস্ট: Rust ৬৪, JS ৮ — সব পাস।

## আপনার করণীয়
1. Supabase SQL Editor-এ `supabase/migrations/20261025010000_preset_lists.sql` চালান (এখনো লাইভে নেই — যাচাই করা)।
2. Chrome Web Store-এ নতুন `dist/content-guard-extension.zip` (1.1.0) আপলোড — ব্রাউজার আলাদা করে চেনার জন্য এটা লাগবে।
3. Certificate: Certum থেকে আপনাকেই কিনতে হবে (আমি পেমেন্ট করতে পারি না)। `docs/EXTENSION-AND-CERTIFICATE-GUIDE.md` দেখুন।

---

# আপডেট — ২৫ অক্টোবর ২০২৬ (বিকেল): লাইভ যাচাই + বড় পরিসরের প্রস্তুতি, v1.2.0

- লাইভ Supabase: `get_presets` ১১টি লিস্ট দেয়; `preset_lists` টেবিল anon key দিয়ে সরাসরি পড়া যায় না (permission denied) ✅। আপনার PC লাইভ লিস্ট সিঙ্ক করেছে ✅।
- Agent-এর খরচ মাপা: এক কোরের ~০.৯% CPU, ~১২ MB মেমোরি।

| # | সিদ্ধান্ত | কারণ |
|---|---|---|
| R27 | লাইসেন্স যাচাই ১৫ মিনিট → **~৬ ঘণ্টা ±২০%**; চালুর পর ৫–১২০ সেকেন্ড এলোমেলো দেরি; Premium পাতা খুললে তখনই (মিনিটে সর্বোচ্চ একবার)। | ১ কোটি PC-তে সেকেন্ডে ~২২,০০০ অনুরোধ হতো → এখন ~৪৬০। |
| R28 | প্রস্তুত লিস্ট শুধু admin বদলালে নামে: লাইসেন্সের উত্তরে `presets_version`। পুরনো server-এ দিনে একবার। | বিপুল ডেটা-খরচ এড়ানো। নতুন SQL: `supabase/migrations/20261025020000_presets_version.sql`। |
| R29 | Version 1.2.0। | — |

পূর্ণ মূল্যায়ন ও এক্সটেনশন আপডেটের নিয়ম: `docs/SCALE-READINESS.md`।
সিদ্ধান্ত: এখন বেটা (১০০–১,০০০ জন) চালানোর মতো; ১ কোটির আগে লাগবে — স্বয়ংক্রিয় আপডেট, code signing, স্বয়ংক্রিয় পেমেন্ট, বেনামি সমস্যা-রিপোর্ট।

---

# আপডেট — ২৫ অক্টোবর ২০২৬ (সন্ধ্যা): ১ কোটির জন্য চারটি ব্যবস্থা — v1.3.0

| # | সিদ্ধান্ত / কাজ | যাচাই |
|---|---|---|
| R30 | **স্বয়ংক্রিয় আপডেট** (`crates/agent/src/update.rs`): লাইসেন্স-উত্তরে `latest_release`; Ed25519 স্বাক্ষর (ring), SHA-256, আকার যাচাই; Program Files-এ নামানো (সাধারণ ইউজার বদলাতে পারে না); নীরব ইনস্টল; ধাপে ধাপে rollout; প্রতি version সর্বোচ্চ ৩ চেষ্টা। গোপন চাবি `C:\Users\user\HTL-Signing\` (প্রজেক্টের বাইরে)। | dev-এ পুরো প্রবাহ ✅; একই স্বাক্ষরে অন্য version (জাল) প্রত্যাখ্যাত ✅; ইউনিট টেস্ট (স্বাক্ষর, rollout, version তুলনা) ✅ |
| R31 | **বেনামি health রিপোর্ট + crash log**: শুধু server জানালে পাঠায় (`accepts_health`), কোনো ব্যক্তিগত/ব্রাউজিং তথ্য নয়। admin view `admin_health_summary`, `admin_recent_problems`। PRIVACY.md আপডেট। | mock server পর্যন্ত ✅ |
| R32 | **পেমেন্ট**: Paddle (merchant of record) webhook, HMAC + replay-সুরক্ষা, idempotent, নবায়নে দিন যোগ হয় (পুরনো বাগ ঠিক)। generic পথ bKash/SSLCommerz-এর জন্য। purchase.html-এ Paddle checkout; ক্রয়-পাতার ঠিকানা server থেকে (`app_config.purchase_url`)। | Deno টেস্ট ৩টি ✅ |
| R33 | **Code signing pipeline** (`build-release.ps1`, NSIS `!finalize`/`!uninstfinalize`, Microsoft signtool NuGet থেকে)। | অস্থায়ী টেস্ট certificate দিয়ে তিনটি ফাইল timestamp-সহ সাইন ✅ (টেস্ট cert পরে মোছা) |
| R34 | Version 1.3.0। | v1.3.0 ইনস্টল UAC বাতিল হওয়ায় এখনো হয়নি; PC-তে 1.2.0 চলছে |

নতুন SQL (ক্রমে): 020000_presets_version, 030000_releases_and_health, 040000_payments।
গাইড: `docs/LAUNCH-SYSTEMS.md`। টেস্ট মোট: Rust ৬৯, JS ৮, Deno ৩।

---

# ⚠️ নিরাপত্তা — ২৫ অক্টোবর ২০২৬ (রাত)
- **লাইভ Supabase-এ গুরুতর ফাঁক পাওয়া গেছে (নিরাপদভাবে যাচাই করা):** `admin_grant_premium` ও `admin_create_code` public (anon) চাবি দিয়ে চালানো যেত — যে কেউ নিজেকে আজীবন Premium দিতে বা কোড বানাতে পারত। কারণ: Supabase প্রতিটি নতুন function anon/authenticated-কে সরাসরি দেয়, আর আগের SQL শুধু `public` থেকে সরিয়েছিল।
- ঠিক করা: `supabase/migrations/20261025050000_security_fix.sql` (+ setup.sql)। নতুন function এখন থেকে ডিফল্টে public নয়।
- সব আজকের SQL (020000–050000) এই PC-তে **আসল PostgreSQL**-এ (pgserver) Supabase-এর মতো role দিয়ে চালিয়ে পরীক্ষা: ২৮টি চেক পাস, দুবার চালানো নিরাপদ।
- পরীক্ষার সময় ধরা পড়ল: Windows-এ psql ঠিকানার পরের option উপেক্ষা করে — পরীক্ষার harness ঠিক করা হয়েছে (আগের ভুয়া "PASS" বাতিল)।

---

# পেমেন্ট ও certificate — খরচের বিশ্লেষণ (২৫ অক্টোবর ২০২৬)
- যাচাই: লাইভ server-এ নিরাপত্তা-ফাঁক বন্ধ (admin function → permission denied) ✅; PC-তে v1.3.0, health চালু ✅; তালাবদ্ধ চাবি Desktop-এ ✅।
- Paddle: ৫% + ৫০¢ প্রতি বিক্রি, মাসিক ফি নেই; $১০-এর কম দামে আলাদা চুক্তি লাগে। → ৳১৯৯ মাসিক প্ল্যানের জন্য অনুপযুক্ত (ফি ~৩৬%)। সিদ্ধান্ত: Paddle শুধু বিদেশি ক্রেতার বার্ষিক/আজীবন প্ল্যান (USD-এ)।
- বাংলাদেশি ক্রেতা (bKash/Nagad/Rocket/ব্যাংক): এখন হাতে-যাচাই ফর্ম (খরচ নেই)। বিক্রি বাড়লে SSLCommerz (৳২৫,৫০০ এককালীন + ২.৫%; ট্রেড লাইসেন্স, TIN, ব্যবসায়িক ব্যাংক অ্যাকাউন্ট লাগে)।
- Certum: ন্যূনতম ১ বছর, মাসিক/ট্রায়াল নেই। €২০৯ ≈ ৳২৯,০০০–৩০,০০০ (€১ ≈ ৳১৩৯)। সিদ্ধান্ত: বেটায় এখনই কেনার দরকার নেই; সবার জন্য ছাড়ার আগে কেনা।

---

# অ্যাপের ভেতর থেকে আপডেট (Android-এর মতো) — v1.4.0
| # | কাজ | যাচাই |
|---|---|---|
| R35 | টেবিল `app_releases`-এ নতুন version → সবার অ্যাপে "🎉 নতুন আপডেট এসেছে — সংস্করণ X / কী নতুন: …" + **"এখনই আপডেট করুন"** বাটন (সব পাতায়), Settings-এ "আপডেট" অংশ (বর্তমান সংস্করণ, আপডেট খুঁজুন)। বাটন চাপলে rollout-এর বাইরে থাকলেও তখনই আপডেট। নিজে থেকেও আপডেট হয় (rollout অনুযায়ী)। সেটিংস অক্ষত (installer ProgramData রেখে দেয়)। ৬ ভাষায়। | dev-এ বাটন → "আপডেট হচ্ছে…" → নামানো+স্বাক্ষর যাচাই ✅ |
| R36 | `notes` / `notes_bn` কলাম + Supabase Storage bucket `releases` (SQL `20261025060000_update_table.sql`)। | আসল PostgreSQL-এ পরীক্ষা ✅ |
| R37 | **এক-কমান্ডের `packaging\windows\publish-update.ps1`**: version পড়ে, "কী নতুন" ছোট জানালায় জিজ্ঞেস করে, সাইন করে, Supabase-এ আপলোড করে, টেবিলে সারি বসায়, অনলাইন ফাইল মিলিয়ে দেখে। Supabase secret key প্রথমবার চায়, Windows DPAPI-তে এনক্রিপ্ট করে রাখে (`HTL-Signing\supabase-secret.txt`)। | dry-run ✅ (আসল আপলোড ব্যবহারকারীর চাবি দিয়ে বাকি) |
| R38 | Version 1.4.0 (`dist`)। | বিল্ড + সব টেস্ট ✅ |

---

# সহজ Premium ব্যবস্থা (SQL ছাড়া, শুধু Table Editor) — v1.4.0-এর অংশ
| # | কাজ | যাচাই |
|---|---|---|
| R39 | অ্যাপের Premium পাতায় ৩ ধাপে কেনা: প্ল্যান → bKash/Nagad/Rocket নম্বর (কপি বাটন) → TrxID → জমা দিন। ডিভাইস আইডি অ্যাপ নিজে পাঠায়। অনুরোধ অপেক্ষমাণ থাকলে অ্যাপ প্রতি ২ মিনিটে যাচাই করে (২ দিন পর্যন্ত)। | dev-এ: জমা → server-এ পৌঁছানো → approve → ২ মিনিটে Premium ✅ |
| R40 | `purchase_requests.status` এখন ড্রপডাউন (pending/approved/rejected); approved বাছলে trigger Premium দেয় (monthly=১ মাস, yearly=১২, lifetime=আজীবন), `result`/`premium_until` নিজে ভরে। একই TrxID দুবার নয়; প্রতি ডিভাইসে ৫টি অপেক্ষমাণ সীমা। | আসল PostgreSQL ✅ |
| R41 | **`premium_grants` টেবিল**: Insert row → Device ID (পুরো বা প্রথম ৮+ অক্ষর) + months (খালি = আজীবন) → Premium চালু; `result`, `premium_until`, `created_at` নিজে ভরে; ভুল আইডিতে ❌ বার্তা। | আসল PostgreSQL ✅ |
| R42 | bKash/Nagad/Rocket নম্বর ও দাম `app_config` টেবিল থেকে অ্যাপে যায় (অ্যাপ আপডেট ছাড়াই বদলানো যায়)। | ✅ |
| R43 | মেয়াদের তারিখ এখন পুরো তারিখে দেখায় (আগে "সোম ৬:৪১ PM" দেখাত — বাগ)। | ✅ |
SQL: `supabase/migrations/20261025060000_update_table.sql` + `20261025070000_easy_premium.sql`।

---

# ২৬ অক্টোবর ২০২৬
- ⚠️ **Supabase CLI ভুল অ্যাকাউন্টে লগইন:** দেখা যায় শুধু `mostafa4344` (lvatrprvhiiqpnprctmc) আর `content-guardAa` (ivpubslnnzrgluhfruoi) — কোনোটাই অ্যাপের server `qhplhhrmathbfyejkzbr` নয়, নামে "PC" নেই। **কিছু চালানো হয়নি।** সঠিক অ্যাকাউন্টে `supabase login` দরকার।
- R44: hosts ফাইলে সর্বোচ্চ ১৫,০০০ নাম (৩০,০০০ লাইন); বড় লিস্টে উপ-ঠিকানা কমে (৯ → www → শুধু মূল), মূল সাইট সবসময় থাকে; হিসাব শুধু লিস্ট বদলালে। কারণ: বড় hosts ফাইলে Windows-এর DNS ধীর হয়। টেস্ট ✅।
- R45: `web/terms.html`, `web/refund.html` (**১৪ দিনের পুরো টাকা ফেরত** — EU ডিজিটাল-পণ্যের মান, Paddle-এর প্রত্যাশা; আপনি চাইলে বদলাবেন), `web/privacy.html` (PRIVACY.md থেকে), `web/faq.html` (বাংলা+ইংরেজি)। **বাকি: সাপোর্টের যোগাযোগ (ইমেইল/WhatsApp) বসানো** — faq.html-এ "Support: —"।

## ২৮ অক্টোবর ২০২৬
- Supabase-এ 060000 + 070000 চালানো হয়েছে (CLI, project "Content guard pc" যাচাই করে)। সব admin ফাংশন anon থেকে বন্ধ যাচাই করা।
- 1.4.0 লাইভ প্রকাশিত (storage `releases/v1.4.0`, `app_releases` rollout 100)। 1.3.0-তে install বোতাম নেই — একবার হাতে installer চালাতে হবে, এরপর থেকে আপডেট নিজে থেকে।
- Premium পাতা: সার্ভারের পেমেন্ট তথ্য আসার আগে "আনা হচ্ছে…" + রিফ্রেশ বোতাম (আগে "যোগাযোগ করুন" দেখাত)।
- সাপোর্ট: WhatsApp 01568689505 (wa.me/8801568689505), contentguard.support@gmail.com — faq/terms/privacy-তে।
- ওয়েবসাইট আপলোড: `dist/website-upload/content-guard-pc/` (+ zip), গাইড `WEBSITE-UPLOAD-GUIDE.html`। ডাউনলোড বোতাম একই ফোল্ডারের Setup.exe-তে।

## ২৮ অক্টোবর ২০২৬ (বিকেল) — টুলটিপ ও আসল-ব্রাউজার পরীক্ষা
- প্রতিটা ফিচারে (i) সাহায্য: `ui/help.js`, ৪২টা, English মূল + বাংলা অনুবাদ; অন্য ভাষায় Google Translate লিংক (শুধু সাহায্যের লেখা যায়)।
- আসল PC-তে পরীক্ষা (`packaging/test/browser-check.mjs`, DevTools দিয়ে): Chrome/Edge/Firefox/Opera-তে পুরো-সাইট ব্লক পাস; অ্যাপ ব্লক ২ সেকেন্ড; Chrome-এ এক্সটেনশন (সাইটের অংশ, শব্দ, ব্লক পাতা) পাস; SafeSearch পাস।
- **ফাঁক ১ (ঠিক):** পুরনো এক্সটেনশন (1.0.0) ব্রাউজারের নাম পাঠায় না → Chrome-এর রিপোর্টে Edge-ও "Protected" দেখাত। এখন নামহীন রিপোর্ট শুধু তখনই গোনা হয় যখন ওই ধরনের ব্রাউজার একটাই চলছে।
- **ফাঁক ২ (ঠিক):** Tor Browser `firefox.exe` নামে চলে → Firefox ভেবে ছেড়ে দিত। এখন প্রোগ্রামের ফোল্ডারে `TorBrowser/Tor/tor.exe` থাকলে নাম যাই হোক "Tor Browser" ধরা হয় ও বন্ধ হয়।
- **ফাঁক ৩ (ঠিক):** "অসমর্থিত ব্রাউজার বন্ধ" আর "এক্সটেনশন ছাড়া ব্রাউজার বন্ধ" এক সুইচে ছিল → Tor আটকাতে গেলে Edge/Firefox-ও বন্ধ হত। এখন দুটো আলাদা; নতুন ইউজারের জন্য প্রথমটা + SafeSearch ডিফল্টে চালু।
- **বাকি (ইউজারের কাজ):** Edge on home PC শুধু Edge Add-ons স্টোরের এক্সটেনশন force-install করে; Firefox-এ AMO লাগে। দুই স্টোরে প্রকাশের পর এজেন্টে Edge/Firefox-এর আলাদা আইডি বসাতে হবে।
- মালিকের PC-তে পরীক্ষার জন্য ১ মাস Premium (premium_grants id 1)।
- Brave (per-user install): এক্সটেনশন policy দিয়ে নিজে বসে, পুরো-সাইট ও সাইটের-অংশ ব্লক পাস। Opera-তে এক্সটেনশন বসে না (Chrome policy মানে না) — পুরো-সাইট ব্লক পাস।
- প্রাইভেট উইন্ডো policy (Chrome/Edge/Brave/Firefox) রেজিস্ট্রিতে বসে; DevTools-এর createBrowserContext policy মানে না, তাই ওই পরীক্ষা অর্থহীন।

## ২৯ অক্টোবর ২০২৬ — প্ল্যান টেবিল, সেটিংস-ফাঁক, ঘড়ি
- **প্ল্যান:** দাম এখন `plans` টেবিলে (Table Editor → plans): `months` (খালি = আজীবন), `price`, ঐচ্ছিক `name`, `popular`, `enabled`, `sort`। নতুন row = নতুন প্ল্যান, অ্যাপ আপডেট লাগে না; কেনা/অনুমোদন ঠিক ততো মাস দেয়। কোড: monthly/yearly/lifetime বা "<n>m"। পুরনো `app_config` price_* row মুছে দেওয়া (এক জায়গায় দাম)। দামের আগে-পিছের ফাঁকা/নতুন-লাইন সার্ভার বাদ দেয়। (migration 20261029000000)
- **সেটিংস-ফাঁক (ঠিক):** সুরক্ষা চালু থাকলে আংশিক-ব্লক ফিল্টার সরানো, DNS ফিল্টার বন্ধ, আর তৈরি লিস্টের preset ট্যাগ মোছা যেত — এখন এজেন্ট আটকায়। UI-তে লক থাকা টিক ধূসর।
- **ব্যাকআপ:** কোড আকারে (`HTLCG1:` + base64) — লুকানো লিস্টের সাইট পড়া যায় না; পুরনো JSON ব্যাকআপও নেয়। রিস্টোর একই লক-নিয়মে যায় (ঢিলা ব্যাকআপ আটকায় — পরীক্ষিত)।
- **ঘড়ি (ঠিক):** (১) রিস্টার্টের পর Windows ঘড়ি বিশ্বাস করা হয় না — শেষ বিশ্বস্ত সময় থেকে চলে যতক্ষণ না সার্ভারের HTTPS `Date` মেলে; (২) টলারেন্স ১২০ সেকেন্ড → ৫ সেকেন্ড + ১% (ধীরে ধীরে ঘড়ি ঠেলা আর কাজ করে না); (৩) টাইম জোন বদল সুরক্ষা চালু থাকলে ২৪ ঘণ্টা পর কার্যকর; (৪) বিশ্বস্ত সময় প্রতি মিনিটে সেভ। অফলাইনে রিস্টার্ট হলে সময় পিছিয়ে থাকে (কড়া দিকে ভুল) যতক্ষণ না ইন্টারনেট আসে।
- আসল PC-তে যাচাই: clock_verified, ২ মাসের প্ল্যান, টাইম জোন টোকিও করে দেখা — অ্যাপ বাংলাদেশ সময়ই রাখে।
- **লোগো:** শর্টকাট আগে `contentguard-agent.exe`-এর আইকন খুঁজত, কিন্তু exe-তে আইকন নেই → ফাঁকা আইকন। এখন ইনস্টলার `appicon.ico` অ্যাপের ফোল্ডারে রাখে; Start menu শর্টকাট, .url আর Apps & features সেটা দেখায় (PC-তে যাচাই)।
- **প্রস্তুত লিস্ট সহজ এডিট:** `preset_lists`-এর sites/apps/keywords এখন সাধারণ লেখা (কমা/নতুন লাইন দিয়ে আলাদা; sites-এ স্পেসও)। সার্ভার `split_entries()` দিয়ে https://, www., শেষের /, উদ্ধৃতি/ব্র্যাকেট, ডুপ্লিকেট বাদ দেয়; পুরনো JSON পেস্টও চলে। লাইভে প্রয়োগের আগে-পরে অ্যাপ হুবহু একই লিস্ট পায়, presets_version বদলায়নি। (migration 20261029010000)
- **খোলা জানালায় পুরনো পাতা:** আপডেটের সময় অ্যাপের জানালা খোলা থাকলে সেটা আগের ভার্সনের পাতা চালিয়ে যেত (যেমন নতুন প্ল্যান দেখাত না)। এখন এজেন্টের ভার্সন বদলালে জানালা নিজে নতুন পাতা লোড করে, একই ট্যাবে থাকে আর "আপডেট সম্পন্ন" দেখায় (ফর্মে টাইপ চলার সময় অপেক্ষা করে)। এই ফিক্সের আগের ভার্সনের খোলা জানালা একবার বন্ধ করে খুলতে হবে।

## ২৯ অক্টোবর ২০২৬ — সহজ বাছাই ও সময় বেছে লক (মাপকাঠি: শতভাগ কার্যকর, আন্তর্জাতিক, সহজ)
- **সাইট বা অ্যাপ বাছাই (`ui/picker.js`):** ফোকাস সেশন, সময়সূচি, দৈনিক সীমা — তিন জায়গায় একই বাছাই: গত ৭ দিনে বেশি ব্যবহৃত সাইট/অ্যাপ, এই PC-তে বসানো অ্যাপ (Start menu শর্টকাট থেকে নাম + আসল প্রোগ্রাম), এখন চলা অ্যাপ, বা যেকোনো ওয়েবসাইট লিখে। প্রতিটা বাছাই নিজের এক-আইটেমের লিস্ট হয় (আবার বাছলে একই লিস্ট ফেরত), তাই লক/সময়সূচি/সীমার সব নিয়ম একই থাকে।
- **আসল প্রোগ্রাম চেনা (`crates/agent/src/apps.rs`):** .lnk পার্সার (MS-SHLLINK)। Squirrel অ্যাপ (Discord, Slack…) `Update.exe --processStart X.exe` দিয়ে চলে → আসল X.exe নেওয়া হয়, নইলে ব্লক কাজ করত না। ব্রাউজার থেকে বসানো ওয়েব-অ্যাপ (chrome_proxy.exe) বাদ — ওগুলো ওয়েবসাইট। "এখন চলছে" তালিকা থেকে সার্ভিস/টুল বাদ।
- **সময় বেছে লক:** "কখন পর্যন্ত…" (শেষের তারিখ-সময়), "পরে…" (৭ দিন আগে পর্যন্ত পরিকল্পনা), নতুন লক "শুধু নির্দিষ্ট সময়ে থামানো যাবে" (দৈনিক জানালা, রাত পেরোলেও চলে)। শুরুর আগে এক লাইনের সারকথা (প্রতিটা ভাষার নিজস্ব দাঁড়ি ও সংখ্যা)।
- **নিরাপত্তা:** পরিকল্পিত লক সেট করার মুহূর্ত থেকেই বাঁধাধরা (`Session::is_locked` = শেষ না হওয়া পর্যন্ত) — নইলে শুরুর এক মিনিট আগে লিস্ট খালি বা সেশন বাতিল করা যেত। সব সময় এজেন্টের বিশ্বস্ত ঘড়ি দিয়ে যাচাই।
- সময়সূচি/সীমার "নতুন" বোতাম আর লিস্ট ছাড়া বন্ধ থাকে না — ফর্মের ভেতর থেকেই সাইট/অ্যাপ বাছা যায়।
- ডেভ অ্যাপে যাচাই: youtube.com লিখে বাছা, CapCut (capcut.exe), "পরে" + "কখন পর্যন্ত" + সময়-জানালা লক, জানালার বাইরে থামাতে বাধা, শুরুর আগে লিস্ট খালি করতে বাধা, Cursor-এ ৪৫ মিনিটের সীমা, netflix.com সময়সূচি। এখনো build/PC-তে ইনস্টল হয়নি।
- **হোয়াইটলিস্ট (ব্যতিক্রম), প্রতিটা লিস্টে:** `allow_sites` (সাইট, সাব-ডোমেইন বা পাতা, যেমন `youtube.com/@khanacademy`, `mail.google.com`) আর `allow_apps` (`minecraft.exe`)। নিয়ম: ব্যতিক্রম শুধু নিজের লিস্টের ব্লক তোলে — একই সময়ে চালু অন্য লিস্ট একই সাইট ব্লক করলে ব্লকই থাকে (`Policy.rules`, লিস্ট ধরে ধরে)। hosts: ব্যতিক্রমের ডোমেইন বাদ পড়ে শুধু যখন অন্য কোনো লিস্ট সেটা ব্লক করে না; পাতা-স্তরের ব্যতিক্রম থাকলে পুরো ডোমেইন hosts থেকে সরে, সিদ্ধান্ত নেয় এজেন্ট/এক্সটেনশন। এক্সটেনশনের তাৎক্ষণিক নিয়মে ব্যতিক্রম-ছোঁয়া সাইট আর ব্যতিক্রমওয়ালা লিস্টের শব্দ যায় না — পুরনো Store এক্সটেনশনও এজেন্টকে জিজ্ঞেস করে ঠিক সিদ্ধান্ত পায়, নতুন এক্সটেনশন প্রকাশ লাগে না। সুরক্ষা চালু থাকলে ব্যতিক্রম শুধু সরানো যায়, যোগ নয়; "*" কখনো ব্যতিক্রম নয়। দৈনিক সীমায় ব্যতিক্রমের সময় গোনা হয় না। ডেভে যাচাই: সিদ্ধান্ত, এক্সটেনশন নিয়ম, hosts ফাইল।
- **দৈনিক সীমা — ৫ মিনিট আগে সতর্কবার্তা (`crates/agent/src/notify.rs`):** এজেন্ট সার্ভিস হিসেবে চলে, নিজের স্ক্রিন নেই, তাই Windows-এর অনুমোদিত `WTSSendMessageW` দিয়ে চালু ইউজারের স্ক্রিনে ছোট বার্তা (৬০ সেকেন্ডে নিজে বন্ধ, এজেন্ট আটকায় না)। দিনে প্রতি সীমায় একবার, আর শুধু যখন সময় আসলেই খরচ হচ্ছে (বাকি সময় কমছে)। ব্যবহারকারীর ভাষায় (৬ ভাষা)। সব ব্রাউজার আর অ্যাপে কাজ করে। PC-তে যাচাই: Notepad-এ ৫ মিনিটের সীমা → "Notepad: আজ আর ৫ মিনিট বাকি…"।
- **কিছু দিনে আলাদা সময় (`Limit.day_minutes`, সোমবার থেকে ৭টা মান):** যেমন স্কুলের দিনে ৩০ মিনিট, ছুটির দিনে ২ ঘণ্টা; ০ = সেদিন সারাদিন ব্লক। খালি থাকলে আগের মতো প্রতিদিন `daily_minutes`। লক/সুরক্ষা চালু থাকলে প্রতিটা দিনের সময় শুধু কমানো যায় (৭ দিনই আলাদা করে যাচাই)। পুরনো সীমা আর পুরনো ভার্সনের সাথে মিল থাকে।
- **সময়সূচির তৈরি অপশন:** নতুন সময়সূচিতে এক চাপে শুরু: অফিস সময় (৯–১৭), স্কুল সময় (৮–১৫), সন্ধ্যার পড়া (১৮–২১), ঘুমের সময় (২৩–০৬), সারাদিন। কাজের দিন ভাষা অনুযায়ী: বাংলা/আরবি = রবি–বৃহস্পতি, অন্যরা = সোম–শুক্র। পরে দিন/সময় বদলানো যায়। কোনো দিন না বাছলে সেভ হয় না। শুরু = শেষ হলে "সারাদিন" দেখায়।
- **"আপনার সপ্তাহ" ছবি:** সময়সূচি পাতার ওপরে ৭ দিনের ২৪ ঘণ্টার দাগে সব চালু সময়সূচি, প্রতিটা লিস্টের আলাদা রং, আজকের সারিতে এখনকার সময়ের দাগ, রাত পেরোনো সময় পরের দিনে। পাতার নিরাপত্তা নিয়ম (CSP) inline style আটকায়, তাই অবস্থান data-* থেকে JS দিয়ে বসানো হয়।
- নামাজের সময় ব্লক: ব্যবহারকারী বাদ দিতে বলেছেন (২৯ অক্টোবর ২০২৬)।

## ২৯ অক্টোবর ২০২৬ — ফাঁক বন্ধ (মাপকাঠি: কোনো বাইপাস নয়, সহজ, জরুরি কাজ বন্ধ নয়)
- **ব্যতিক্রম যোগ = সুরক্ষা অনুরোধ (`RequestKind::AddExceptions`):** লিস্ট লক বা সুরক্ষা চালু থাকলে নতুন ব্যতিক্রম সাথে সাথে যোগ হয় না; সেভ করলে বাকি বদল সেভ হয়, আর নতুন ব্যতিক্রম একটা অনুরোধ হয়ে যায় — আনইনস্টল/সুরক্ষা কমানোর মতোই অপেক্ষা (মনোযোগী: চ্যালেঞ্জ + ৬০ সে, প্রতিশ্রুতিবদ্ধ: বেছে নেওয়া সময়, পার্টনার)। অনুমোদনের পর "প্রয়োগ" চাপলে যোগ হয়। সরানো সবসময় সাথে সাথে। সুরক্ষা বন্ধ কিন্তু লিস্ট লক সেশন/লক সময়সূচিতে আটকানো হলে অনুরোধ নেওয়া হয় না (সেশন শেষ হওয়া পর্যন্ত) — নইলে সাথে সাথে অনুমোদিত অনুরোধ দিয়ে লক এড়ানো যেত। সরাসরি API দিয়ে যোগ করার চেষ্টা `list_locked`। ডেভে পুরো পথ যাচাই।
- **পাতা-স্তরের ব্যতিক্রমের ফাঁক:** `youtube.com/@channel` পুরো youtube.com-কে hosts থেকে সরায়, তাই এক্সটেনশন ছাড়া ব্রাউজারে পুরো সাইট খুলে যেত। এখন `Policy.needs_extension` — এমন লিস্ট চালু থাকলে এক্সটেনশন ছাড়া সব ব্রাউজার বন্ধ হয়। সাব-ডোমেইন ব্যতিক্রম (mail.google.com) hosts-এ ঠিক থাকে, তাই সেখানে লাগে না।
- **সবসময় পাহারা:** Chrome, Brave, Vivaldi, Chromium-এ এক্সটেনশন নিজে বসে (নীতি), তাই সেখানে এক্সটেনশন না থাকলে = কেউ সরিয়েছে → ব্লকের সময় বন্ধ। Edge/Firefox-এ এখনো স্টোর আইডি নেই, তাই সেগুলো জোর করে বন্ধ হয় না (জরুরি কাজ যেন না আটকায়) — যদি না ব্যবহারকারী নিজে সুইচ চালু করেন বা পাতা-ব্যতিক্রম চালু থাকে।
- **৯০ সেকেন্ডের সতর্কতা সবসময়:** আগে অনেকক্ষণ খোলা ব্রাউজার সাথে সাথে বন্ধ হতো। এখন বন্ধের আগে স্ক্রিনে বার্তা ("Edge ৯০ সেকেন্ড পর বন্ধ হবে…") আর পুরো ৯০ সেকেন্ড। (ডেভ পরীক্ষায় এই সমস্যাটা ধরা পড়ে: ডেভ এজেন্ট এক্সটেনশনের রিপোর্ট পায় না, তাই PC-র খোলা Chrome/Edge বন্ধ করেছিল।)
- **নতুন ইনস্টলের ডিফল্ট:** এক্সটেনশন স্থায়ীভাবে বসানো আর প্রাইভেট উইন্ডো বন্ধ — চালু।
- **Secure DNS বন্ধ:** সুরক্ষা চালু বা SafeSearch থাকলে Chrome/Edge/Brave/Chromium/Vivaldi-তে `DnsOverHttpsMode=off`, Firefox-এ `DNSOverHTTPS` Enabled=0, Locked=1 — নাম খোঁজা সবসময় Windows দিয়ে যায়। আনইনস্টলে (`cleanup`) সরে যায়।
- **Safe Mode:** সার্ভিস `SafeBoot\Minimal` ও `SafeBoot\Network`-এ নিবন্ধিত, তাই Safe Mode-এও ব্লক চলে। আনইনস্টলে সরে যায়। (রেজিস্ট্রি কমান্ডের গঠন HKCU পরীক্ষা-কী-তে যাচাই; আসল HKLM লেখা ইনস্টল করা ভার্সনে যাচাই বাকি।)
- **নাম বদলানো প্রোগ্রাম:** Windows ফাইলের ভেতরের `OriginalFilename` পড়ে — chrome.exe-কে notes.exe নামে চালালেও ব্রাউজার হিসেবে ধরা পড়ে, আর ব্লক করা অ্যাপের নাম বদলালেও ব্লক হয়। (Notepad কপি করে নাম বদলে পরীক্ষা।)
- **1.5.0 আসল PC-তে যাচাই (২৯ অক্টোবর ২০২৬, ইনস্টলারের আপডেট-পথ, সুরক্ষা প্রতিশ্রুতিবদ্ধ ২৪ ঘণ্টা):** সার্ভিস চালু ও নিজে আবার চালু হয়; SafeBoot Minimal/Network নিবন্ধিত; ৫টা Chromium ব্রাউজারে DnsOverHttpsMode=off, প্রাইভেট উইন্ডো বন্ধ, এক্সটেনশন forcelist; Firefox DoH Enabled=0/Locked=1, প্রাইভেট বন্ধ। Chrome (Store এক্সটেনশন): youtube ভিডিও ব্লক, @khanacademy খোলা, reddit ব্লক, old.reddit খোলা, উইকিপিডিয়া খোলা, প্রাপ্তবয়স্ক সাইট ব্লক। Edge (এক্সটেনশন নেই) পাতা-ব্যতিক্রম চালু অবস্থায়: ২ সেকেন্ডে বার্তা, ৯৬ সেকেন্ডে বন্ধ। Firefox: reddit "Problem loading page" (0.0.0.0), old.reddit আর উইকিপিডিয়া খোলা। notepad কপি করে mytool.exe নামে চালালে ১ সেকেন্ডে বন্ধ। লক লিস্টে সরাসরি ব্যতিক্রম `list_locked`; অনুরোধ ঠিক ২৪ ঘণ্টা পরে খোলে (বাতিল করা হয়েছে)। পরীক্ষার লিস্ট/সেশন মুছে ফেলা হয়েছে।
- **লক্ষ করার মতো:** Store-এর পুরনো এক্সটেনশন (1.0) শুধু "chromium" পরিবার রিপোর্ট করে — Chrome আর Edge একসাথে খোলা থাকলে কোনটায় এক্সটেনশন আছে বোঝা যায় না, আর Chrome বন্ধের পর ৯০ সেকেন্ড পর্যন্ত Edge-কে সুরক্ষিত ধরা হয়। 1.1.0 (ব্রাউজার আলাদা করে রিপোর্ট করে) Store-এ আপলোড হলে মিটবে।
- **ব্রাউজার বন্ধ শুধু সুইচ দিয়ে (৩০ অক্টোবর ২০২৬, মালিকের নির্দেশ):** এক্সটেনশন না চললে ব্রাউজার বন্ধ হবে কিনা তা শুধু সুরক্ষা পাতার "এক্সটেনশন কাজ না করলে ব্রাউজার বন্ধ" সুইচ (`require_browser_extension`) ঠিক করে। আগের স্বয়ংক্রিয় পাহারা (Chrome/Brave/Vivaldi সবসময়, পাতা-ব্যতিক্রমের সময় সব ব্রাউজার) সরানো হয়েছে। সুইচ বন্ধ থাকলে কোনো ব্রাউজার বন্ধ হয় না, সাধারণ ফিল্টার (hosts, SafeSearch) চলে। ফাঁক যেন না খোলে: সুইচ বন্ধ থাকলে পাতা-স্তরের ব্যতিক্রম (`youtube.com/@x`) নিষ্ক্রিয় থাকে (`engine::active_allow_sites`), পুরো সাইট hosts-এ ব্লক থাকে; সাব-ডোমেইন ব্যতিক্রম (mail.google.com) সবসময় কাজ করে। Tor-এর মতো অসমর্থিত ব্রাউজার বন্ধ করা আলাদা সুইচ।
- **নিরাপত্তা: লাইসেন্স চুরি/ক্র্যাক ঠেকানো (1.6.0, ৩০ অক্টোবর ২০২৬):**
  - **স্বাক্ষর করা লাইসেন্স:** সার্ভার প্রতিটা উত্তরে Ed25519-এ সই করা টোকেন দেয় (`license_register` / `license_check` / `license_redeem`, মাইগ্রেশন `20261030000000_signed_licenses.sql`)। গোপন চাবি শুধু Supabase Vault-এ (`license_signing_seed`), অ্যাপে শুধু পাবলিক অংশ (`LICENSE_PUBLIC_KEY_HEX` = `dbac7c21…e8b8`)। অ্যাপ প্ল্যান/ট্রায়াল/মেয়াদ শুধু বৈধ সই থেকে নেয়; license.json হাতে বদলানো, সার্ভার ব্লক করা বা ভুয়া সার্ভার দিয়ে Premium পাওয়া যায় না (dev-এ আক্রমণ করে যাচাই: বদলানো ফাইল → `locked_offline`, নতুন কিছু যোগ 402)। **এই Vault secret কখনো মুছবেন/বদলাবেন না** — বদলালে সব ইনস্টল করা অ্যাপের Premium বন্ধ হবে।
  - **কম্পিউটারে বাঁধা:** প্রতিটা PC-তে একটা গোপন চাবি (`device.key`, Windows DPAPI মেশিন-স্কোপে সিল, শুধু SYSTEM/Administrators পড়তে পারে)। সার্ভার প্রথমবার এর ছাপ (`accounts.device_key`) রাখে; টোকেনে সেই ছাপ থাকে। ডিভাইস আইডি বা লাইসেন্স ফাইল অন্য PC-তে কপি করলে টোকেন মেলে না, সার্ভারও টোকেন দেয় না (`device_mismatch`)। **সাপোর্ট:** কেউ Windows/অ্যাপ নতুন করে বসিয়ে "আরেকটা কম্পিউটারের সাথে যুক্ত" দেখালে Table Editor → accounts → সেই ডিভাইসের `device_key` ঘর খালি করুন; পরের যাচাইয়ে নতুন PC যুক্ত হবে।
  - **প্রথম চালু, ইন্টারনেট নেই:** সই করা লাইসেন্স একবারও না পেলে সার্ভিস চালুর পর ১৫ মিনিট সেটআপ করা যায়, তারপর সার্ভারের সই লাগবে।
  - **বন্ধ করা ফাঁক:** setup.sql/seed.sql-এ লেখা পরীক্ষার কোড (PREMIUM-LIFETIME ইত্যাদি) লাইভ সার্ভারে চালু ছিল — মুছে দেওয়া হয়েছে, ফাইল থেকেও সরানো। `purchase_requests`-এ সরাসরি লেখার anon নীতি সরানো, accounts/codes/purchase_requests থেকে anon টেবিল-অধিকার বাতিল, পুরনো `redeem_code` বন্ধ; নতুন `license_redeem`-এ ভুল কোডের সীমা (ডিভাইসে ঘণ্টায় ১০, IP-তে ৩০)।
  - **ডেটা ফোল্ডার:** Windows-এর ডিফল্টে ProgramData-তে সাধারণ (অ্যাডমিন নয়) ইউজারও ফাইল বানাতে পারত — ফলে বাচ্চার অ্যাকাউন্ট `config.json.tmp` আগে বানিয়ে রেখে সেটিংস ফাইলের মালিক হয়ে ব্লক তুলে দিতে পারত। এখন সার্ভিস চালুর সময় ফোল্ডার শুধু SYSTEM/Administrators লিখতে পারে (Users শুধু পড়ে), ভেতরের সবকিছুর মালিক SYSTEM; আর সেভ করার সময় পুরনো tmp ফাইল মুছে নতুন করে বানানো হয়।
  - **রিলিজ বিল্ড:** symbol/debug তথ্য ছাড়া (`strip`), পুরো প্রোগ্রাম অপ্টিমাইজ (`lto`) — পড়া ও প্যাচ করা কঠিন।
  - **যা পুরোপুরি ঠেকানো যায় না (সব সফটওয়্যারেই):** কেউ নিজের PC-র .exe ফাইল নিজে প্যাচ করলে। কোড-সাইন সার্টিফিকেটও এটা আটকায় না। প্রতিরোধ: অটো-আপডেট প্যাচ করা ফাইল বদলে দেয়, আর **GitHub রিপো পাবলিক থাকলে যে কেউ লাইসেন্স-চেক ছাড়া নিজে বিল্ড করে নিতে পারে — রিপো প্রাইভেট করা সবচেয়ে জরুরি।**
