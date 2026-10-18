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
id pgocjgiigaakaplebgnfdbcdfaemkcnd) — এগুলো বদলালে এখনকার ইনস্টল + extension id
ভাঙত। ZIP পুনরায় বানানো হয়েছে; ব্যবহারকারী Package ট্যাবে নতুন ZIP re-upload করবে
(একই store item, একই id)।
