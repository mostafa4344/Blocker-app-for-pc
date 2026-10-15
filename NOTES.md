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
