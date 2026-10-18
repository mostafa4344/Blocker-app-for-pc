# Purchase পেজ — সেটআপ ও অর্ডার পূরণ (HTL Content Guard PC)

`web/purchase.html` = আপনার সাইটে অ্যাপের নিচে বসানোর "Premium কিনুন" পেজ।
ক্রেতা bKash/Nagad-এ টাকা পাঠিয়ে ফর্ম পূরণ করে → অনুরোধ Supabase-এ জমা হয় →
আপনি যাচাই করে Premium চালু করে দেন।

## ১. Supabase টেবিল বানান (একবার)
Supabase → SQL Editor-এ গিয়ে `supabase/purchase-requests.sql` ফাইলের পুরো
লেখা পেস্ট করে **Run** করুন। (আগে `setup.sql` চালানো থাকতে হবে।)

## ২. HTML-এ আপনার তথ্য বসান
`web/purchase.html` খুলে উপরের `CONFIG` অংশে বদলান:
```
BKASH: "01XXXXXXXXX"   ← আপনার bKash Personal নম্বর
NAGAD: "01XXXXXXXXX"   ← আপনার Nagad Personal নম্বর
PRICES: { ... }        ← দাম ইচ্ছেমতো
```
(`SUPABASE_URL` ও `SUPABASE_KEY` আগে থেকেই বসানো — ওই publishable key
ক্লায়েন্টে দেওয়া নিরাপদ, এটা দিয়ে কেউ ডেটা পড়তে পারে না, শুধু অনুরোধ জমা দিতে পারে।)

## ৩. পেজটা আপনার সাইটে বসান
`purchase.html`-এর পুরো লেখা আপনার সাইটে অ্যাপের নিচে HTML হিসেবে বসিয়ে দিন
(অথবা পুরো ফাইলটা halaltechlife.com-এ আপলোড করে লিংক দিন)।

## ৪. অর্ডার পূরণ (প্রতিটি কেনাকাটায়)
1. Supabase → **Table editor → `purchase_requests`** — নতুন সারি (status `pending`) দেখুন।
2. ক্রেতার `method` + `txn_id` দিয়ে আপনার bKash/Nagad-এ টাকা এসেছে কিনা মিলিয়ে নিন।
3. মিললে, SQL Editor-এ ক্রেতার `install_id` দিয়ে Premium চালু করুন:
   ```sql
   -- lifetime (আজীবন):
   select public.admin_grant_premium('<install_id>', null);
   -- monthly (৩০ দিন):   select public.admin_grant_premium('<install_id>', 30);
   -- yearly (৩৬৫ দিন):   select public.admin_grant_premium('<install_id>', 365);
   ```
   ক্রেতার অ্যাপ পরের sync-এ নিজে থেকেই Premium হয়ে যাবে (কোড লাগবে না)।
4. চাইলে কোড পাঠাতে পারেন (install_id না জানলে):
   ```sql
   select public.admin_create_code('PREMIUM-XXXX', null, 'order note');
   ```
   ক্রেতা অ্যাপের Premium পেজে এই কোড বসাবে।
5. সারিটার `status` কলাম `approved` করে দিন (রেকর্ডের জন্য)।

> admin RPC দুটো শুধু **service_role** key দিয়ে চলে — তাই এগুলো শুধু
> Supabase ড্যাশবোর্ড/SQL Editor থেকে চালাবেন, কখনো পাবলিক পেজে key দেবেন না।

## পরে: স্বয়ংক্রিয় গেটওয়ে
পেজে "স্বয়ংক্রিয় কার্ড/গেটওয়ে — শীঘ্রই" অংশটা রাখা আছে। Stripe/SSLCommerz
অ্যাকাউন্ট হলে ওখানে বোতাম যুক্ত করে পুরো প্রক্রিয়া স্বয়ংক্রিয় করা যাবে
(webhook → `admin_grant_premium`); template `supabase/functions/payment-webhook`-এ আছে।
