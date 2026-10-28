# চারটি বড় ব্যবস্থা — কীভাবে চালু ও ব্যবহার করবেন (v1.3.0)

এক নজরে:
| ব্যবস্থা | কোড | পরীক্ষা | আপনার করণীয় |
|---|---|---|---|
| ১. স্বয়ংক্রিয় আপডেট | ✅ তৈরি | ✅ dev-এ পুরো প্রবাহ + নকল রিলিজ প্রত্যাখ্যান | SQL চালানো, গোপন চাবির ব্যাকআপ |
| ২. বেনামি সমস্যা-রিপোর্ট | ✅ তৈরি | ✅ dev-এ server পর্যন্ত | SQL চালানো |
| ৩. স্বয়ংক্রিয় পেমেন্ট | ✅ তৈরি | ✅ স্বাক্ষর-যাচাইয়ের টেস্ট | Paddle অ্যাকাউন্ট + ওয়েবসাইট |
| ৪. Code signing | ✅ তৈরি | ✅ টেস্ট certificate দিয়ে পুরো pipeline | Certum certificate কেনা |

## প্রথমে: Supabase-এ ৩টি SQL চালান (ক্রমানুসারে)
SQL Editor → প্রতিটি paste → Run:
1. `supabase/migrations/20261025020000_presets_version.sql`
2. `supabase/migrations/20261025030000_releases_and_health.sql`
3. `supabase/migrations/20261025040000_payments.sql`

(অথবা টার্মিনালে একবার `supabase login` আর `supabase link --project-ref qhplhhrmathbfyejkzbr`
চালালে আমি নিজেই চালিয়ে দিতে পারব।)

---

## ১. স্বয়ংক্রিয় আপডেট

### কীভাবে কাজ করে
- অ্যাপ লাইসেন্স যাচাইয়ের সাথেই জেনে নেয় নতুন version আছে কিনা (বাড়তি অনুরোধ লাগে না)।
- নতুন হলে নামিয়ে **আকার, SHA-256 আর আপনার ডিজিটাল স্বাক্ষর** মিলিয়ে দেখে। মিললে নীরবে ইনস্টল করে
  (protection চালু থাকলেও; সেটিংস অক্ষত থাকে)।
- **ধাপে ধাপে ছাড়া:** `rollout` = কত শতাংশ ডিভাইস পাবে। ১% দিয়ে শুরু, সমস্যা না হলে ১০ → ৫০ → ১০০।
- কোনো version ৩ বার ব্যর্থ হলে সেই ডিভাইস আর চেষ্টা করে না (অবিরাম লুপ হয় না)।

### 🔑 গোপন চাবি — খুব গুরুত্বপূর্ণ
- ফাইল: `C:\Users\user\HTL-Signing\update-signing-key.pk8`
- **আজই একটা USB/নিরাপদ জায়গায় কপি রাখুন।** এটা হারালে আর কোনো আপডেট পাঠানো যাবে না
  (সবাইকে হাতে নতুন installer চালাতে হবে)।
- **কাউকে দেবেন না, GitHub-এ তুলবেন না।** যার কাছে এটা থাকবে সে সব ইউজারের PC-তে প্রোগ্রাম পাঠাতে পারবে।
- অ্যাপের ভেতরে শুধু public চাবি আছে: `9290048a…3bbc` — এটা গোপন নয়।

### নতুন version ছাড়ার ধাপ
1. আমি কোড বদলে version বাড়াই, তারপর:
   `powershell -ExecutionPolicy Bypass -File packaging\windows\build-release.ps1 [-CertThumbprint …]`
2. `dist\HTL-Content-Guard-PC-Setup.exe` কোনো https ঠিকানায় তুলুন — প্রস্তাব: **GitHub Releases**
   (বিনামূল্যে, বিশাল ডাউনলোড সামলায়)। ⚠️ Supabase Storage-এ রাখবেন না: ১ কোটি × ৩.৭ MB = প্রতি রিলিজে ~৩৭ TB ডাউনলোড,
   খরচ হাজার ডলার।
3. `powershell -ExecutionPolicy Bypass -File packaging\windows\make-release.ps1 -Version 1.3.1 -Url <ঠিকানা>`
   → `dist\release-1.3.1.sql` তৈরি হয়।
4. সেই SQL Supabase-এ চালান (rollout ১% দিয়ে শুরু)।
5. ১–২ দিন পর Table Editor → `app_releases` → `rollout` বাড়ান। কোনো সমস্যা হলে `enabled = false`।

---

## ২. বেনামি সমস্যা-রিপোর্ট
- অ্যাপ প্রতিটি লাইসেন্স যাচাইয়ের সাথে ছোট বেনামি রিপোর্ট পাঠায় (কোনো সাইট/অ্যাপ/নাম নয় — `PRIVACY.md` আপডেট করা হয়েছে)।
- অ্যাপ crash করলে কারণটা পরের বার একবার পাঠায়।
- **দেখবেন কোথায়:** Supabase → Table Editor বা SQL Editor:
  - `admin_health_summary` — কোন version কতজন, কতজনের ব্লকিং ভাঙা, crash, আপডেট ব্যর্থ, Premium
  - `admin_recent_problems` — সমস্যাওয়ালা ডিভাইসগুলোর বিস্তারিত (শেষ ৫০০)
- নতুন version ছাড়ার পর `admin_health_summary` দেখে নিশ্চিত হন সব ঠিক আছে, তারপর rollout বাড়ান।

---

## ৩. স্বয়ংক্রিয় পেমেন্ট (Paddle)
**কেন Paddle:** বিশ্বজুড়ে কার্ড/PayPal নেয়, প্রতিটি দেশের VAT/ট্যাক্স নিজে জমা দেয় (merchant of record),
বাংলাদেশি বিক্রেতাকে টাকা পাঠায়। (Stripe বাংলাদেশে খোলা যায় না।)

### আপনার ধাপ
1. **ওয়েবসাইট:** Paddle একটা ওয়েবসাইট চায় — পণ্যের বিবরণ, দাম, Terms, Privacy, Refund policy সহ।
   `web/purchase.html` এর অংশ। সহজ উপায়: GitHub Pages বা নিজের ডোমেইন।
2. **Paddle অ্যাকাউন্ট** খুলুন (paddle.com → Sign up; পরিচয় যাচাই লাগে)। প্রথমে **sandbox** দিয়ে পরীক্ষা।
3. Paddle-এ ৩টি price বানান (Monthly / Yearly / Lifetime) → প্রতিটির `pri_…` id নিন।
4. Paddle → Developer tools → **Notifications** → নতুন destination:
   `https://qhplhhrmathbfyejkzbr.supabase.co/functions/v1/payment-webhook?provider=paddle`,
   event: `transaction.completed` → secret key কপি।
5. আমাকে জানালে আমি deploy করব (অথবা আপনি):
   ```
   supabase functions deploy payment-webhook --no-verify-jwt
   supabase secrets set PADDLE_WEBHOOK_SECRET=<পেমেন্ট secret> PRODUCT_DAYS='{"pri_মাসিক":30,"pri_বার্ষিক":365,"pri_আজীবন":null}'
   ```
6. `web/purchase.html`-এর উপরের `CONFIG.PADDLE`-এ client token আর price id বসান।
7. Supabase → `app_config` টেবিলে `purchase_url` = আপনার ক্রয়-পাতার ঠিকানা। তখন সব ইউজারের অ্যাপের Premium পাতায়
   **"Premium নিন"** বাটন দেখাবে (ডিভাইস আইডি নিজে থেকেই বসে যায়)। কোনো app update লাগে না।

### নিরাপত্তা
- Paddle-এর স্বাক্ষর (HMAC) যাচাই হয়; পুরনো বার্তা আবার পাঠিয়ে (replay) Premium নেওয়া যায় না।
- একই পেমেন্ট দুবার এলে দুবার দেয় না (`payments` টেবিল)।
- আগে নবায়ন করলে বাকি দিন হারায় না (আগে এটা হারাত — বাগ ঠিক করা হয়েছে)।
- `admin_recent_payments` view-এ সব পেমেন্ট দেখা যায়।
- bKash/Nagad হাতে-যাচাইয়ের পুরনো ফর্ম আগের মতোই চলবে। পরে SSLCommerz/bKash গেটওয়ে নিলে
  `?provider=generic` পথ তৈরি আছে।

---

## ৪. Code signing
- Pipeline তৈরি ও পরীক্ষিত: agent exe, uninstaller আর Setup.exe তিনটাই timestamp-সহ সাইন হয়।
- Microsoft-এর অফিসিয়াল `signtool` রাখা আছে `C:\Users\user\tools\sdk-buildtools\`।
- Certum certificate কেনার পর:
  1. SimplySign Desktop চালু করে ফোনের কোড দিয়ে সংযোগ করুন।
  2. thumbprint দেখুন: `Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Format-List Subject, Thumbprint`
  3. `build-release.ps1 -CertThumbprint <thumbprint>` চালান — বাকিটা স্বয়ংক্রিয়।
- ক্রম মনে রাখুন: **আগে Authenticode সাইন (build-release), তারপর আপডেট-স্বাক্ষর (make-release)**।
