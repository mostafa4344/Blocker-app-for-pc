# Tutorial video script: one entry per scene.
#   shot   : image in shots/ (frame decides how it is shown)
#   frame  : app | web | store | block | installer | dialog | title | end
#   focus  : mark to zoom on (None = whole screen), zoom = how close (1 = whole)
#   text   : English narration; [[key]] = moment the mark `key` gets a ring,
#            [[key!]] = a cursor moves there and clicks.
#   bn     : Bangla subtitle lines, one per English sentence.
# Words in SAY are spoken differently from how they are written.

SAY = {"bKash": "bee-Kash", "Nagad": "Nogod", "TrxID": "transaction ID"}

CHAPTERS = [
    ("intro", "Welcome"),
    ("download", "1 · Download"),
    ("install", "2 · Install"),
    ("setup", "3 · First-time setup"),
    ("ext", "4 · Browser extension"),
    ("lists", "5 · Block lists"),
    ("focus", "6 · Focus sessions"),
    ("sched", "7 · Schedules & daily limits"),
    ("safe", "8 · SafeSearch & Shorts"),
    ("prot", "9 · Protection"),
    ("premium", "10 · Free trial & Premium"),
    ("help", "Help & support"),
]

S = [
  # ---------------- intro ----------------
  dict(ch="intro", frame="title", shot=None,
       text="Welcome to HTL Content Guard PC, the content blocker that helps you stay focused and protected on your Windows computer.",
       bn=["HTL Content Guard PC-তে আপনাকে স্বাগতম — Windows কম্পিউটারে মনোযোগী ও নিরাপদ থাকার কনটেন্ট ব্লকার।"]),
  dict(ch="intro", frame="app", shot="home_premium", focus=None,
       text="In this video, I'll show you every step: how to download and install it, set it up in one minute, block websites, apps and keywords, and how the free trial and Premium work.",
       bn=["এই ভিডিওতে প্রতিটি ধাপ দেখাব: ডাউনলোড ও ইনস্টল, এক মিনিটে সেটআপ, ওয়েবসাইট-অ্যাপ-কীওয়ার্ড ব্লক, আর ফ্রি ট্রায়াল ও Premium কীভাবে কাজ করে।"]),

  # ---------------- download ----------------
  dict(ch="download", frame="web", shot="web_hero", focus="dl", zoom=1.5,
       text="First, open our website and click [[dl!]]Download free for Windows. It's a single file of about two and a half megabytes, so there's nothing to unzip.",
       bn=["প্রথমে আমাদের ওয়েবসাইট খুলে 'Download free for Windows' চাপুন।", "এটা প্রায় আড়াই মেগাবাইটের একটাই ফাইল, তাই unzip করার কিছু নেই।"]),
  dict(ch="download", frame="web", shot="web_trial", focus="box", zoom=1.3,
       text="[[box]]Every feature is free for four days. You don't need a card, a payment, or even an account to start.",
       bn=["৪ দিন সব ফিচার একদম ফ্রি।", "শুরু করতে কার্ড, টাকা বা কোনো অ্যাকাউন্ট লাগে না।"]),

  # ---------------- install ----------------
  dict(ch="install", frame="dialog", shot="ss1",
       text="Open the downloaded file. Because we're a new publisher, Windows may show a blue box called Windows protected your PC. This is normal for new apps. Click [[btn!]]More info.",
       bn=["ডাউনলোড করা ফাইলটি খুলুন।", "আমরা নতুন প্রকাশক, তাই Windows একটা নীল বক্স দেখাতে পারে: 'Windows protected your PC'।", "নতুন অ্যাপের ক্ষেত্রে এটা স্বাভাবিক।", "'More info' চাপুন।"]),
  dict(ch="install", frame="dialog", shot="ss2",
       text="Then click [[btn!]]Run anyway.",
       bn=["তারপর 'Run anyway' চাপুন।"]),
  dict(ch="install", frame="dialog", shot="uac", dim=True,
       text="Windows now asks if this app may make changes to your device. Click [[btn!]]Yes. The app needs this permission to block websites for every browser on your computer.",
       bn=["এবার Windows জানতে চাইবে অ্যাপটি পরিবর্তন করতে পারবে কিনা।", "'Yes' চাপুন।", "কম্পিউটারের সব ব্রাউজারে ওয়েবসাইট ব্লক করতে অ্যাপটির এই অনুমতি দরকার।"]),
  dict(ch="install", frame="installer", shot="inst1_welcome", btn=(0.645, 0.905, 0.145, 0.055),
       text="The setup window opens. Click [[btn!]]Next.",
       bn=["সেটআপ উইন্ডো খুলবে।", "'Next' চাপুন।"]),
  dict(ch="install", frame="installer", shot="inst2_directory", btn=(0.645, 0.905, 0.145, 0.055),
       text="Keep the suggested folder, and click [[btn!]]Install.",
       bn=["প্রস্তাবিত ফোল্ডারই রাখুন, আর 'Install' চাপুন।"]),
  dict(ch="install", frame="installer", shot="inst3_progress",
       text="Installation takes just a few seconds.",
       bn=["ইনস্টল হতে মাত্র কয়েক সেকেন্ড লাগে।"]),
  dict(ch="install", frame="installer", shot="inst4_finish", btn=(0.645, 0.905, 0.145, 0.055),
       text="Leave Open HTL Content Guard PC now ticked, and click [[btn!]]Finish. You can always open the app again from the Start menu.",
       bn=["'Open HTL Content Guard PC now' টিক দেওয়া রেখে 'Finish' চাপুন।", "পরে Start মেনু থেকেও অ্যাপটি খুলতে পারবেন।"]),

  # ---------------- first-time setup ----------------
  dict(ch="setup", frame="app", shot="ob1", focus="lang", zoom=1.6,
       text="The app opens with a one-minute setup. [[lang]]Choose your language. The app speaks English, Bengali, Arabic, Urdu, Indonesian and Turkish.",
       bn=["অ্যাপটি এক মিনিটের সেটআপ দিয়ে খুলবে।", "আপনার ভাষা বেছে নিন।", "অ্যাপটি ইংরেজি, বাংলা, আরবি, উর্দু, ইন্দোনেশীয় ও তুর্কি ভাষায় চলে।"]),
  dict(ch="setup", frame="app", shot="ob1_typed", focus="goal", zoom=1.6,
       text="Then write your goal. [[goal]]It's shown on the block page to remind you why you started. Click [[next!]]Next.",
       bn=["তারপর আপনার লক্ষ্য লিখুন।", "কেন শুরু করেছিলেন, মনে করিয়ে দিতে এটা ব্লক পেজে দেখানো হয়।", "'Next' চাপুন।"]),
  dict(ch="setup", frame="app", shot="ob2", focus="grid", zoom=1.35,
       text="Now pick what you want to block. [[adult]]Adult content and [[gambling]]gambling are blocked around the clock. [[shorts]]Short videos, like Shorts and Reels, are selected too.",
       bn=["এবার কী ব্লক করবেন বেছে নিন।", "অ্যাডাল্ট কনটেন্ট আর জুয়া ২৪ ঘণ্টা ব্লক থাকে।", "Shorts ও Reels-এর মতো ছোট ভিডিওও বাছাই করা আছে।"]),
  dict(ch="setup", frame="app", shot="ob2_social", focus="social", zoom=1.35,
       text="Tap any other list you want, like [[social!]]Social media. You can change all of this later. Click [[next!]]Next.",
       bn=["আর যা চান বেছে নিন, যেমন 'Social media'।", "এগুলো পরে যেকোনো সময় বদলাতে পারবেন।", "'Next' চাপুন।"]),
  dict(ch="setup", frame="app", shot="ob3", focus="mindful", zoom=1.45,
       text="Finally, choose how strict to be with yourself. [[off]]Off lets you change anything at any time. [[mindful]]Mindful, which we recommend, asks you to type a long text and wait one minute before you can lower protection. [[committed]]Committed makes you wait a full day, or longer if you choose.",
       bn=["শেষে ঠিক করুন নিজের প্রতি কতটা কঠোর হবেন।", "'Off' দিলে যেকোনো সময় সব বদলানো যায়।", "'Mindful' (আমাদের পরামর্শ) দিলে সুরক্ষা কমাতে একটা লম্বা লেখা টাইপ করে এক মিনিট অপেক্ষা করতে হয়।", "'Committed' দিলে পুরো এক দিন, বা আপনার বেছে নেওয়া আরও বেশি সময় অপেক্ষা করতে হয়।"]),
  dict(ch="setup", frame="app", shot="ob3", focus="finish", zoom=1.45,
       text="For your first seven days, practice mode keeps every lock to two hours at most, so you can't trap yourself by mistake. Click [[finish!]]Finish.",
       bn=["প্রথম ৭ দিন 'practice mode' চালু থাকে, কোনো লক ২ ঘণ্টার বেশি থাকে না — তাই ভুল করে নিজেকে আটকে ফেলার ভয় নেই।", "'Finish' চাপুন।"]),
  dict(ch="setup", frame="app", shot="home_first", focus="main", zoom=1.25,
       text="That's it. [[main]]Your protection is already on, and the home screen shows what's being blocked right now.",
       bn=["ব্যস, হয়ে গেল।", "আপনার সুরক্ষা এখনই চালু, আর হোম স্ক্রিনে দেখা যায় এই মুহূর্তে কী ব্লক হচ্ছে।"]),

  # ---------------- extension ----------------
  dict(ch="ext", frame="app", shot="settings_browsers", focus="managed", zoom=1.4,
       text="Websites are already blocked in every browser. The free browser extension adds even more: it hides Shorts and Reels, blocks pages by keyword, and shows a friendly block page. [[managed]]Setup has already turned on Install the extension permanently, so the extension adds itself to Chrome, Brave and Vivaldi.",
       bn=["সব ব্রাউজারে ওয়েবসাইট আগে থেকেই ব্লক হয়।", "ফ্রি ব্রাউজার এক্সটেনশন আরও বেশি করে: Shorts ও Reels লুকায়, কীওয়ার্ড দিয়ে পেজ ব্লক করে, আর সুন্দর একটা ব্লক পেজ দেখায়।", "সেটআপেই 'Install the extension permanently' চালু হয়ে গেছে, তাই Chrome, Brave ও Vivaldi-তে এক্সটেনশন নিজে থেকেই বসে যায়।"]),
  dict(ch="ext", frame="store", shot="store", focus="add", focusbox=(0.12, 0.13, 0.76, 0.14), zoom=1.3,
       text="In Microsoft Edge, open the Chrome Web Store, search for HTL Content Guard PC, and click [[add!]]Add to Chrome. If Edge asks, allow extensions from other stores.",
       bn=["Microsoft Edge-এ Chrome Web Store খুলে 'HTL Content Guard PC' খুঁজুন, তারপর 'Add to Chrome' চাপুন।", "Edge জিজ্ঞেস করলে অন্য স্টোরের এক্সটেনশন অনুমতি দিন।"]),
  dict(ch="ext", frame="app", shot="home_ext_on", focus=None,
       text="When the extension is connected, the yellow warning on the home screen disappears.",
       bn=["এক্সটেনশন যুক্ত হলে হোম স্ক্রিনের হলুদ সতর্কবার্তা চলে যায়।"]),

  # ---------------- lists ----------------
  dict(ch="lists", frame="app", shot="lists", focus="preset", zoom=1.4,
       text="Open Block lists. A list holds the websites, apps and words you want to block. Use [[preset!]]Add a ready-made list for categories we keep up to date for you.",
       bn=["'Block lists' খুলুন।", "একটা লিস্টে থাকে যে ওয়েবসাইট, অ্যাপ আর শব্দ আপনি ব্লক করতে চান।", "আমাদের নিয়মিত আপডেট করা ক্যাটাগরির জন্য 'Add a ready-made list' ব্যবহার করুন।"]),
  dict(ch="lists", frame="app", shot="lists_presets", focus="grid", zoom=1.3,
       text="Social media, video streaming, games, chat apps, news, shopping, dating apps, and even AI homework helpers.",
       bn=["সোশ্যাল মিডিয়া, ভিডিও স্ট্রিমিং, গেম, চ্যাট অ্যাপ, খবর, শপিং, ডেটিং অ্যাপ — এমনকি AI হোমওয়ার্ক হেল্পারও।"]),
  dict(ch="lists", frame="app", shot="list_new", focus="sites", zoom=1.35,
       text="Or click New list to make your own. Give it a name, then add [[sites]]websites, one per line, [[apps]]programs, like a game, and [[words]]words. Any page whose address contains one of these words is blocked. Click [[save!]]Save.",
       bn=["অথবা 'New list' চেপে নিজের লিস্ট বানান।", "একটা নাম দিন, তারপর ওয়েবসাইট (প্রতি লাইনে একটা), প্রোগ্রাম (যেমন কোনো গেম) আর শব্দ যোগ করুন।", "যে পেজের ঠিকানায় এই শব্দগুলোর কোনোটা থাকবে, সেটা ব্লক হবে।", "'Save' চাপুন।"]),
  dict(ch="lists", frame="app", shot="lists_after", focus=None,
       text="Lists marked Always on are blocked twenty-four seven. The others are blocked when you start a focus session, or by a schedule or a daily limit.",
       bn=["'Always on' লেখা লিস্টগুলো ২৪ ঘণ্টা, ৭ দিন ব্লক থাকে।", "বাকিগুলো ব্লক হয় যখন আপনি ফোকাস সেশন চালু করেন, বা সময়সূচি কিংবা দৈনিক সময়সীমা অনুযায়ী।"]),

  # ---------------- focus ----------------
  dict(ch="focus", frame="app", shot="focus_setup", focus="summary", zoom=1.3,
       text="To focus right now, go to Home. Pick the lists to block, choose [[dur!]]how long, and how the session can be stopped. [[summary]]The summary tells you exactly what will happen. Click [[start!]]Start.",
       bn=["এখনই মনোযোগ দিতে Home-এ যান।", "কোন লিস্ট ব্লক হবে, কতক্ষণ, আর সেশন কীভাবে থামানো যাবে — বেছে নিন।", "নিচের সারাংশে ঠিক কী হবে তা লেখা থাকে।", "'Start' চাপুন।"]),
  dict(ch="focus", frame="app", shot="focus_running", focus="main", zoom=1.3,
       text="[[main]]Blocking now shows each list and exactly when it ends.",
       bn=["'Blocking now'-এ প্রতিটি লিস্ট আর ঠিক কখন শেষ হবে তা দেখা যায়।"]),
  dict(ch="focus", frame="block", shot="blockpage", focus="title", zoom=1.4,
       text="If you open a blocked site, you'll see this page instead, [[goal]]with your goal, to keep you on track.",
       bn=["ব্লক করা সাইট খুললে তার বদলে এই পেজটা দেখবেন — সাথে আপনার লক্ষ্য, যেন আপনি পথে থাকেন।"]),

  # ---------------- schedules & limits ----------------
  dict(ch="sched", frame="app", shot="sched_new", focus="tpl", zoom=1.4,
       text="Schedules block a list automatically on the days and hours you choose. Start from a quick template, like [[tpl!]]Work hours, pick the [[days]]days, and set the [[start]]start and end time. Click [[save!]]Save.",
       bn=["সময়সূচি আপনার বেছে নেওয়া দিন ও সময়ে নিজে থেকেই লিস্ট ব্লক করে।", "'Work hours'-এর মতো একটা টেমপ্লেট দিয়ে শুরু করুন, দিন বাছুন, শুরু আর শেষের সময় দিন।", "'Save' চাপুন।"]),
  dict(ch="sched", frame="app", shot="sched_after", focus=None,
       text="Your week is shown at a glance.",
       bn=["পুরো সপ্তাহ এক নজরে দেখা যায়।"]),
  dict(ch="sched", frame="app", shot="limit_new", focus="min", zoom=1.45,
       text="Daily limits give you a set amount of time each day. For example, [[min]]thirty minutes of short videos. When the time is used up, it's blocked until tomorrow. Click [[save!]]Save.",
       bn=["দৈনিক সময়সীমা প্রতিদিন নির্দিষ্ট পরিমাণ সময় ব্যবহার করতে দেয়।", "যেমন, ছোট ভিডিওর জন্য দিনে ৩০ মিনিট।", "সময় শেষ হলে পরদিন পর্যন্ত ব্লক থাকে।", "'Save' চাপুন।"]),

  # ---------------- safe search & partial ----------------
  dict(ch="safe", frame="app", shot="settings_safe", focus="safe", zoom=1.5,
       text="In Settings, [[safe]]Force SafeSearch keeps Google, Bing, YouTube and DuckDuckGo in safe mode in every browser. [[private]]You can also turn off private windows, the most common way around a filter.",
       bn=["Settings-এ 'Force SafeSearch' সব ব্রাউজারে Google, Bing, YouTube ও DuckDuckGo-কে নিরাপদ মোডে রাখে।", "প্রাইভেট উইন্ডোও বন্ধ করতে পারেন — ফিল্টার এড়ানোর সবচেয়ে সাধারণ পথ।"]),
  dict(ch="safe", frame="app", shot="settings_partial", focus="yt", zoom=1.45,
       text="With Partial blocking, a site stays useful without the endless scroll. [[yt!]]Hide YouTube Shorts, the home feed, or comments, the Facebook feed and Reels, Instagram Reels, and more. Remember to click Save.",
       bn=["'Partial blocking' দিয়ে সাইট কাজের থাকে, কিন্তু অন্তহীন স্ক্রল থাকে না।", "YouTube Shorts, হোম ফিড বা কমেন্ট, Facebook ফিড ও Reels, Instagram Reels আর আরও অনেক কিছু লুকাতে পারেন।", "শেষে 'Save' চাপতে ভুলবেন না।"]),

  # ---------------- protection ----------------
  dict(ch="prot", frame="app", shot="protection2", focus="mindful", zoom=1.35,
       text="The Protection page stops you from deleting the app or loosening blocks in a weak moment. [[mindful]]You can switch levels here. Making things stricter works at once. Making them easier always needs the waiting time you chose.",
       bn=["'Protection' পেজ দুর্বল মুহূর্তে অ্যাপ মুছে ফেলা বা ব্লক ঢিলা করা থেকে আপনাকে আটকায়।", "এখানে লেভেল বদলাতে পারবেন।", "কঠোর করা সঙ্গে সঙ্গে কাজ করে।", "সহজ করতে সবসময় আপনার বেছে নেওয়া অপেক্ষার সময় লাগে।"]),
  dict(ch="prot", frame="app", shot="protection_uninstall", focus="req", zoom=1.5,
       text="To uninstall, click [[req!]]Request uninstall. When your level's rule is met, Windows lets you remove the app as usual. Every level keeps a safe way out, so nobody is ever locked in.",
       bn=["আনইনস্টল করতে 'Request uninstall' চাপুন।", "আপনার লেভেলের শর্ত পূরণ হলে Windows থেকে স্বাভাবিকভাবে অ্যাপটি সরাতে পারবেন।", "প্রতিটি লেভেলে নিরাপদে বের হওয়ার পথ থাকে — কেউ কখনো আটকে যায় না।"]),

  # ---------------- premium ----------------
  dict(ch="premium", frame="app", shot="trial_banner", focus="banner", zoom=1.5,
       text="[[banner]]The top of the app shows how many days of your free trial are left. When the trial ends, every block you set keeps working. You only need Premium to add new blocks or make them stronger.",
       bn=["অ্যাপের ওপরে দেখা যায় ফ্রি ট্রায়ালের আর কত দিন বাকি।", "ট্রায়াল শেষ হলেও আপনার সেট করা সব ব্লক চলতে থাকে।", "শুধু নতুন ব্লক যোগ করতে বা আরও শক্ত করতে Premium লাগে।"]),
  dict(ch="premium", frame="app", shot="premium_plan", focus="plan", zoom=1.35,
       text="To buy, open the Premium page and choose [[plan!]]a plan.",
       bn=["কিনতে 'Premium' পেজ খুলে একটা প্ল্যান বেছে নিন।"]),
  dict(ch="premium", frame="app", shot="premium_pay", focus="num", zoom=1.5,
       text="Choose bKash or Nagad. In your payment app, use Send Money to send the amount shown to [[num]]this number. You can [[copy!]]copy it with one click.",
       bn=["bKash বা Nagad বেছে নিন।", "আপনার পেমেন্ট অ্যাপে 'Send Money' দিয়ে দেখানো টাকা এই নম্বরে পাঠান।", "এক ক্লিকেই নম্বরটা কপি করতে পারবেন।"]),
  dict(ch="premium", frame="app", shot="premium_trx", focus="trx", zoom=1.5,
       text="After paying, you'll get a TrxID in the payment message. [[trx]]Type it here, add your mobile number if you like, and click [[send!]]Submit.",
       bn=["টাকা পাঠানোর পর মেসেজে একটা TrxID পাবেন।", "সেটা এখানে লিখুন, চাইলে মোবাইল নম্বর দিন, তারপর 'Submit' চাপুন।"]),
  dict(ch="premium", frame="app", shot="premium_active", focus="status", zoom=1.5,
       text="Once we confirm your payment, usually within a few hours, [[status]]Premium turns on by itself. There's nothing else to do. And every purchase has a fourteen-day money-back guarantee.",
       bn=["আমরা পেমেন্ট নিশ্চিত করলেই (সাধারণত কয়েক ঘণ্টার মধ্যে) Premium নিজে থেকেই চালু হয়ে যায়।", "আপনাকে আর কিছু করতে হবে না।", "আর প্রতিটি কেনাকাটায় ১৪ দিনের মানি-ব্যাক গ্যারান্টি আছে।"]),

  # ---------------- help ----------------
  dict(ch="help", frame="app", shot="premium_devid", focus="dev", zoom=1.5,
       text="Need help? [[dev]]Copy your Device ID from the bottom of the Premium page and send it to us on WhatsApp or by email. We're happy to help.",
       bn=["সাহায্য দরকার?", "'Premium' পেজের নিচ থেকে আপনার Device ID কপি করে আমাদের WhatsApp বা ইমেইলে পাঠান।", "আমরা সানন্দে সাহায্য করব।"]),
  dict(ch="help", frame="end", shot=None,
       text="Thank you for watching. Stay focused, and stay protected.",
       bn=["দেখার জন্য ধন্যবাদ।", "মনোযোগী থাকুন, নিরাপদ থাকুন।"]),
]
