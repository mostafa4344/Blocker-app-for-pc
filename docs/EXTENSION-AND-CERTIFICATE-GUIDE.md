# এক্সটেনশন আপডেট ও Code Signing Certificate — বিস্তারিত গাইড

তৈরি: ২৫ অক্টোবর ২০২৬। দাম ও শর্ত এই তারিখে অফিসিয়াল পাতা থেকে যাচাই করা; কেনার আগে আবার দেখে নিন।

---

## অংশ ১ — Chrome Web Store-এ এক্সটেনশন আপডেট

### এখনকার অবস্থা (যাচাই করা)
- এক্সটেনশন **ইতিমধ্যে Chrome Web Store-এ অনুমোদিত ও লাইভ**। ID: `amlkhggimheblaboilcbjdjomlceckdp`, লাইভ version **1.0.0**।
  (Google-এর অফিসিয়াল update সার্ভার `status="ok"` দিয়েছে।)
- তাই নতুন listing, $5 ফি বা privacy-র প্রশ্নোত্তর আর লাগবে না — শুধু নতুন version আপলোড।

### কেন 1.1.0 আপলোড করা দরকার
| 1.0.0-তে যা নেই | 1.1.0-তে আছে |
|---|---|
| "সবসময় ব্লক" কারণ — ব্লক পেজে কারণের জায়গা খালি দেখায় | ব্লক পেজে "সবসময় ব্লক করা" লেখা |
| শুধু বাংলা/ইংরেজি | আরবি, উর্দু, ইন্দোনেশিয়ান, তুর্কি (আরবি/উর্দুতে ডান-থেকে-বাম) |
| — | popup-ও ৬ ভাষায় |

### ধাপগুলো (প্রায় ৫ মিনিট)
1. https://chrome.google.com/webstore/devconsole খুলে আগের Google অ্যাকাউন্ট দিয়ে ঢুকুন।
2. **HTL Content Guard PC** আইটেমে ক্লিক করুন।
3. বাম মেনু → **Package** → **Upload new package** → `dist/content-guard-extension.zip` দিন।
   (এই zip এখন সঠিক `/` path দিয়ে বানানো; আগের PowerShell-এর `\` সমস্যা নেই।)
4. **Submit for review**। সাধারণত ১–৩ দিন। অনুমোদনের পর সব ইউজারের ব্রাউজার নিজে থেকে আপডেট নেবে — কাউকে কিছু করতে হবে না।
5. নতুন permission যোগ হয়নি, তাই permission-এর ব্যাখ্যা আবার লিখতে হবে না।

### এখনই যা চালু করা যায় (এক্সটেনশন লাইভ, তাই)
- অ্যাপে **Settings → ব্রাউজার সুরক্ষা → "এক্সটেনশন ব্রাউজারে স্থায়ীভাবে বসিয়ে দিন"** চালু করলে Chrome, Edge, Brave, Vivaldi, Chromium-এ এক্সটেনশন নিজে থেকে বসে যাবে আর মুছে ফেলা যাবে না।
  ⚠️ Protection চালু থাকায় একবার চালু করলে আর বন্ধ করা যাবে না (protection কমানোর অনুরোধ + অপেক্ষা ছাড়া)। তাই আগে নিশ্চিত হয়ে চালু করুন।
- এক্সটেনশন বসার পর "এক্সটেনশন ছাড়া কোনো ব্রাউজার চলতে দেবে না" সুইচও চালু করা যাবে।

### পরের ধাপের কাজ (আমি করতে পারব)
1. **অ্যাপের ভেতরের নির্দেশনা বদলানো:** এখন লেখা আছে "Developer mode → Load unpacked"। এক্সটেনশন লাইভ, তাই এর জায়গায় Chrome Web Store-এর সরাসরি লিংক বা "এক ক্লিকে যুক্ত করুন" বাটন দেওয়া উচিত।
2. **Firefox:** এখন Firefox-এ এক্সটেনশন জোর করে বসানো যায় না (signed XPI নেই)। addons.mozilla.org-এ বিনামূল্যে আপলোড করে (unlisted হলেও চলে) signed XPI-এর লিংক পেলে আমি অ্যাপে বসিয়ে দেব।
3. **Edge Add-ons (ঐচ্ছিক, বিনামূল্যে):** Edge এখনই Chrome Web Store থেকে জোর করে বসাতে পারে। আলাদা listing শুধু Edge store-এ খুঁজে পাওয়ার সুবিধার জন্য।
4. **Opera:** Opera প্রতিষ্ঠান-নীতি মানে না, তাই সেখানে জোর করে বসানো সম্ভব নয়। ইউজার নিজে বসাতে পারেন; "এক্সটেনশন ছাড়া ব্রাউজার চলতে দেবে না" সুইচটা ফাঁকটা ঢেকে দেয়।

---

## অংশ ২ — সবচেয়ে কম খরচে অফিসিয়াল Code Signing Certificate

### আগে জেনে রাখুন (Microsoft-এর অফিসিয়াল তথ্য, আগস্ট ২০২৬)
- Certificate থাকলে installer-এ **"Unknown publisher"-এর জায়গায় আপনার নাম** দেখায়, আর antivirus-এর সন্দেহ অনেক কমে।
- **কিন্তু কোনো certificate-ই প্রথম দিন থেকে SmartScreen-এর নীল সতর্কতা পুরো সরায় না।** ২০২৪ থেকে দামি EV certificate-ও OV-র মতোই আচরণ করে। একই certificate দিয়ে প্রতিটি version সাইন করতে থাকলে আর ডাউনলোড বাড়লে "reputation" জমে, তখন সতর্কতা চলে যায়।
  → তাই **EV ($৪০০+)-এর পেছনে টাকা খরচ করার দরকার নেই**; সবচেয়ে সস্তা OV/Standard যথেষ্ট।
- জুন ২০২৩ থেকে certificate-এর গোপন key রাখতে হয় hardware token বা cloud HSM-এ (নিয়ম)। ফেব্রুয়ারি–মার্চ ২০২৬ থেকে একটা certificate সর্বোচ্চ ~৪৫৯ দিন বৈধ; ২–৩ বছরের প্যাকেজে মাঝে বিনামূল্যে নতুন করে ইস্যু হয়।

### আপনার জন্য কোনটা চলবে না
| অপশন | কেন চলবে না |
|---|---|
| **Microsoft Azure Artifact Signing** (মাসে ~$৯.৯৯ — সবচেয়ে সস্তা) | ব্যক্তি (individual) হিসেবে শুধু **USA ও Canada**-র মানুষ পারেন। প্রতিষ্ঠান হিসেবে USA, Canada, EU, UK, অস্ট্রেলিয়া, নিউজিল্যান্ড, জাপান, দক্ষিণ কোরিয়া, সিঙ্গাপুর, সুইজারল্যান্ড, নরওয়ে, ইসরায়েল। বাংলাদেশ এই তালিকায় নেই। |
| **Certum Open Source** (€২৫–৬৯) | শুধু বিনামূল্যের/ওপেন সোর্স সফটওয়্যারের জন্য। আপনার অ্যাপে Premium বিক্রি হয়, তাই এটা দিয়ে সাইন করলে certificate **বাতিল (revoke)** হয়ে যাবে। |
| **SignPath Foundation** (বিনামূল্যে) | শুধু ওপেন সোর্স প্রজেক্টের জন্য। |
| **EV certificate** | দামি, আর SmartScreen-এ এখন বাড়তি কোনো সুবিধা নেই। |

### আপনার জন্য চলবে — দামের তুলনা
| অপশন | দাম (প্রথম বছর) | key কোথায় থাকে | মন্তব্য |
|---|---|---|---|
| ⭐ **Certum Standard Code Signing in the Cloud** (পোল্যান্ডের অফিসিয়াল CA, Microsoft-স্বীকৃত) | **€২০৯** (shop.certum.eu) | Cloud (SimplySign) — মোবাইল অ্যাপের কোড দিয়ে সাইন | **প্রস্তাবিত।** কিছু পাঠাতে হয় না, ব্যক্তি হিসেবে কেনা যায়। মাসে ৫,০০০ সাইন পর্যন্ত (আমাদের লাগবে কয়েকটা)। |
| Certum Standard Code Signing — set | €১৬৯ | USB card reader + smart card (পোল্যান্ড থেকে পাঠানো হয়) | €৪০ সস্তা, কিন্তু বাংলাদেশে কুরিয়ার, কাস্টমস আর দেরির ঝামেলা। কার্ড হারালে সমস্যা। |
| Sectigo OV (reseller দিয়ে) | ~$২১৬–২২৬ | USB token (পাঠানো হয়) | মূলত প্রতিষ্ঠানের জন্য; token পাঠানোর ঝামেলা। |
| SSL.com Individual Validation | $২৪৯ + eSigner cloud $১৮০/বছর | Cloud বা YubiKey | মোট ~$৪৩০ — অনেক দামি। |

> Certum-এর আমেরিকান store (certum.store) একই জিনিস $২৪৯ দেখায়; ইউরোপের অফিসিয়াল shop (shop.certum.eu) €২০৯ — সেখান থেকে কিনুন।
> তৃতীয় পক্ষের কিছু reseller $৯৯–১৬৭ লেখে; সেগুলো অফিসিয়াল Certum পাতায় যাচাই করা যায়নি, তাই সুপারিশ করছি না।

### প্রস্তাবিত পথ: Certum Standard Code Signing in the Cloud — ধাপে ধাপে
1. **কিনুন:** https://shop.certum.eu → Code Signing → **Standard Code Signing in the Cloud** → ১ বছর (বা ২–৩ বছর, প্রতি বছরে সস্তা পড়ে)। কার্ডে পেমেন্ট (আন্তর্জাতিক কার্ড লাগবে)।
2. **কাগজপত্র (ব্যক্তি হিসেবে):**
   - পাসপোর্ট **অথবা** জাতীয় পরিচয়পত্র (NID) / ড্রাইভিং লাইসেন্স
   - আপনার নামে একটা **ইউটিলিটি বিল** (বিদ্যুৎ/গ্যাস/পানি) — ঠিকানার প্রমাণ
   - পরিচয় যাচাই: অনলাইনে স্বয়ংক্রিয়ভাবে (প্রস্তাবিত), অথবা আইডি হাতে নিয়ে নিজের ছবি তুলে। নোটারি লাগে না।
   - ⚠️ Certificate-এ যে নাম যাবে, ইনস্টলারে ইউজাররা সেটাই দেখবেন। **ব্যক্তিগত নাম** যাবে, "Halal Tech Life" নয়। "Halal Tech Life" নামে দেখাতে চাইলে নিবন্ধিত প্রতিষ্ঠান (ট্রেড লাইসেন্স ইত্যাদি) দিয়ে প্রতিষ্ঠান হিসেবে আবেদন করতে হবে।
3. **সেটআপ:** ফোনে **SimplySign** অ্যাপ, PC-তে **SimplySign Desktop** ইনস্টল। সাইন করার সময় ফোনের অ্যাপের কোড দিয়ে সংযোগ হয়।
4. **সাইন করা:** certificate পেলে আমাকে জানান। আমি বিল্ড-প্রক্রিয়ায় `signtool` দিয়ে সাইন করা যুক্ত করব — agent-এর exe, uninstaller ও Setup.exe তিনটাই, timestamp সহ (timestamp থাকলে certificate মেয়াদ শেষ হলেও পুরনো সাইন বৈধ থাকে)।
5. **Reputation দ্রুত বাড়াতে (বিনামূল্যে, অফিসিয়াল):** প্রতিটি নতুন সাইন করা Setup.exe Microsoft-এর https://www.microsoft.com/wdsi/filesubmission পাতায় "software developer" হিসেবে জমা দিন, যাতে Defender ভুল করে আটকে না দেয়।

### খরচের হিসাব (প্রস্তাবিত পথে)
- প্রথম বছর: **€২০৯ (~$২২৫)**। বছরে একবার নবায়ন (২–৩ বছরের প্যাকেজ নিলে প্রতি বছরে কম পড়ে)।
- বাড়তি কোনো hardware বা মাসিক ফি নেই।

### ভবিষ্যতের সস্তা বিকল্প
- নিজের প্রতিষ্ঠান কখনো USA/UK/EU-তে নিবন্ধন করলে **Azure Artifact Signing (মাসে $৯.৯৯ ≈ বছরে $১২০)** ব্যবহার করা যাবে — সবচেয়ে সস্তা আর স্বয়ংক্রিয়।
- **Microsoft Store (MSIX)** দিয়ে বিতরণ করলে Microsoft নিজেই বিনামূল্যে সাইন করে, আর SmartScreen সতর্কতাও আসে না। তবে আমাদের অ্যাপ Windows service + hosts ফাইল ব্যবহার করে, যেটা MSIX-এ বিশেষ অনুমতি (restricted capability) আর Store-এর আলাদা পর্যালোচনা চায়। বড় কাজ — পরের ধাপের জন্য রাখা ভালো।

---

## সূত্র (অফিসিয়াল পাতা)
- Microsoft — Code signing options: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options
- Microsoft — Artifact Signing quickstart (দেশের তালিকা): https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart
- Certum shop (দাম): https://shop.certum.eu/code-signing.html
- Certum — Standard in the Cloud: https://certum.store/standard-code-signing-in-the-cloud.html
- Certum — প্রয়োজনীয় কাগজপত্র: https://support.certum.eu/en/code-signing-required-documents/
- SSL.com eSigner দাম: https://www.ssl.com/guide/esigner-pricing-for-code-signing/
