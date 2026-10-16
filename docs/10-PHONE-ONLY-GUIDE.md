# 10 — ফোন থেকেই সবকিছু করার গাইড (Android phone only)

কম্পিউটার ছাড়া, শুধু আপনার Android ফোন দিয়ে পুরো সিস্টেম চালু করার ধাপ। সব
লেখা আপনার GitHub repo-তে আছে — ফোনের ব্রাউজারে repo খুলে কপি করবেন।

> যা লাগবে: ফোনের একটা ব্রাউজার (Chrome), আপনার Supabase অ্যাকাউন্ট, আপনার
> GitHub অ্যাকাউন্ট। Firebase/Play Store লাগবে না শুরুতে।

---

## ধাপ ১ — ডেটাবেস তৈরি (Supabase, ব্রাউজারে, ~৩ মিনিট)

1. ফোনের ব্রাউজারে **supabase.com** → আপনার প্রজেক্টে ঢুকুন (`eihaavfsupquzdxcixjb`)।
2. বাঁদিকের মেনু → **SQL Editor** → **New query**।
3. আরেকটা ট্যাবে GitHub-এ এই ফাইল খুলুন:
   `backend/supabase/setup_all.sql` → **Raw** বাটনে চাপুন → পুরো লেখা select all →
   copy।
4. SQL Editor-এ paste → **Run**। নিচে "Success" দেখালে টেবিল তৈরি হয়ে গেছে।
5. (ঐচ্ছিক, কিন্তু ভালো) একইভাবে `backend/supabase/retention.sql` copy করে Run করুন
   (পুরোনো ডেটা অটো-মুছে ফেলার জন্য)।

**যাচাই:** SQL Editor-এ এটা Run করুন —
```sql
select table_name from information_schema.tables
where table_schema='public' order by 1;
```
১০টা নাম এলে (children, devices, policies … ) ঠিক আছে।

---

## ধাপ ২ — Edge Function deploy (Supabase dashboard, ব্রাউজারে)

CLI লাগবে না — ড্যাশবোর্ড থেকেই করবেন। **৫টা function** বানাতে হবে। প্রতিটার জন্য:

1. Supabase → বাঁমেনু → **Edge Functions** → **Create a function** (বা "Deploy a
   new function" → via Editor)।
2. Function-এর নাম দিন **হুবহু** এই নামে (একটা একটা করে):
   - `enroll-device`
   - `create-enrollment-token`
   - `device-sync`
   - `device-heartbeat`
   - `push-command`
3. এডিটরে যে ডিফল্ট কোড থাকে সেটা মুছে, GitHub থেকে ওই নামের ফাইলটা copy করে paste করুন:
   `backend/supabase/functions-dashboard/<নাম>/index.ts` (Raw → select all → copy)।
   ⚠️ অবশ্যই **`functions-dashboard`** ফোল্ডারেরটা নেবেন (এগুলো এক-ফাইলে পুরো কোড;
   সাধারণ `functions/` ফোল্ডারেরটা CLI-এর জন্য, ড্যাশবোর্ডে চলবে না)।
4. **Deploy** চাপুন। পাঁচটার জন্য ১–৪ বার করুন।

### ধাপ ২ক — Function secrets (একবারই)
Edge Functions → **Secrets** (Manage secrets) → নিচেরগুলো যোগ করুন:

| Name | Value |
|---|---|
| `DEVICE_JWT_SECRET` | `dfd4f102263961d3f4085048369f04694c7f1c287daddecab124bfdcc49bbcf6` |
| `ENROLLMENT_TOKEN_TTL_MINUTES` | `15` |
| `DEVICE_JWT_TTL_DAYS` | `30` |

> (এই DEVICE_JWT_SECRET আমি আপনার জন্য random বানিয়ে দিয়েছি। চাইলে নিজের একটা
> লম্বা random লেখাও দিতে পারেন। `SUPABASE_URL`, `SUPABASE_ANON_KEY`,
> `SUPABASE_SERVICE_ROLE_KEY` Supabase নিজেই দেয় — ওগুলো দিতে হবে না।)
> `FCM_SERVER_KEY` এখন লাগবে না (push পরে)।

---

## ধাপ ৩ — Parent ওয়েবসাইট চালানো (ফোন থেকে)

ওয়েবসাইটটা হোস্ট করার সবচেয়ে সহজ ফোন-ফ্রেন্ডলি উপায় — **Vercel** (ব্রাউজারে):

1. ফোনে **vercel.com** → GitHub দিয়ে sign in → **Add New Project** → আপনার
   `Blocker-app-for-pc` repo বেছে নিন।
2. **Root Directory** = `parent-web` সেট করুন।
3. **Environment Variables**-এ যোগ করুন:
   - `VITE_SUPABASE_URL` = `https://eihaavfsupquzdxcixjb.supabase.co`
   - `VITE_SUPABASE_ANON_KEY` = `sb_publishable_swMjJ38v3rCDAL-MffNfBw_z_f2ULBu`
   - `VITE_FUNCTIONS_URL` = `https://eihaavfsupquzdxcixjb.supabase.co/functions/v1`
4. **Deploy**। কয়েক মিনিট পর একটা `https://...vercel.app` লিংক পাবেন — সেটাই আপনার
   parent ওয়েবসাইট। ওখানে sign up করে বাচ্চা যোগ করুন।

> Vercel না চাইলে Netlify-ও একইভাবে কাজ করে (Base directory = `parent-web`,
> Build = `npm run build`, Publish = `parent-web/dist`)।

---

## ধাপ ৪ — Android অ্যাপ (APK) ফোনে ইনস্টল

Android Studio লাগবে না — GitHub নিজেই APK বানিয়ে দেবে:

1. GitHub-এ repo → উপরে **Actions** ট্যাব → বাঁয়ে **Android APK** workflow।
2. প্রতিবার কোড push হলে এটা নিজে চলে। সবুজ ✓ হলে ওই run-এ ঢুকুন → নিচে
   **Artifacts** → **familyguard-debug-apk** ডাউনলোড করুন (একটা zip)।
3. ফোনে zip খুলে ভেতরের `app-debug.apk` ট্যাপ করুন → "unknown apps install" অনুমতি
   দিন → **Install**।
4. অ্যাপ খুলুন → disclosure স্ক্রিন → **Start setup** → QR scanner।
5. parent ওয়েবসাইটে (ধাপ ৩) বাচ্চার পাশে **+ Enroll device** → QR দেখান → ফোন দিয়ে
   scan করুন। Enroll হয়ে গেলে persistent notification আসবে।
6. **Finish setup** স্ক্রিনে Usage access / Display over apps / Notifications —
   তিনটা অনুমতি দিন (✓ হবে)।

> টেস্টের জন্য একই ফোনে parent সাইট (ব্রাউজারে) আর child অ্যাপ — দুটোই চালাতে পারেন।
> বাস্তবে child অ্যাপ বাচ্চার ফোনে থাকবে, parent সাইট আপনার ফোনে।

---

## ধাপ ৫ — Play Store (এখন ঐচ্ছিক)

নিজে/পরিবারে ব্যবহার করতে Play Store লাগে না — ধাপ ৪-এর APK sideload করলেই চলে।
সর্বসাধারণের জন্য ছাড়তে চাইলে পরে: Google Play Console ($25, একবার), app signing,
Data Safety form (`docs/08`), privacy-policy লিংক (`docs/07` একটা পাবলিক URL-এ),
তারপর review। এটা বড় ধাপ — পরে ধরা যাবে।

---

## আটকে গেলে
- ধাপ ১/২-এ লাল error এলে — মেসেজটা আমাকে পাঠাবেন, ঠিক করে দেব।
- ধাপ ৪-এ Actions লাল (build fail) হলে — আমি log দেখে কোড ঠিক করব (প্রথম real build-এ
  টুকটাক ঠিক করা লাগতে পারে, স্বাভাবিক)।
