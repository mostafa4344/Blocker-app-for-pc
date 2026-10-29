// Detailed "how this works" help for every feature, shown from the small (i)
// buttons next to each feature. English is the source; a hand translation is
// offered when one exists for the app language, and any language can be
// opened in Google Translate (only this help text is sent, never user data).
import { getLanguage, langInfo } from "/i18n.js";

/** Each entry: t = title, p = paragraphs, how = steps, note = good to know. */
export const HELP = {
  en: {
    home_status: {
      t: "Blocking status",
      p: ["This card shows what is being blocked right now and why. Green means at least one list is active; gray means nothing is blocked at this moment.",
        "Each line shows the list name, the reason (Always blocked, Focus session, Schedule or Daily limit), and when it ends. 🔒 means it is locked and can't be switched off early."],
      note: "Lists marked \"Always blocked\" (for example adult content) stay blocked 24/7 and never end.",
    },
    quick_start: {
      t: "Focus session",
      p: ["A focus session blocks the lists you choose for a set amount of time, starting right now. Use it when you want to study, work or sleep without distractions."],
      how: ["Tap one or more lists (for example Social media). A filled chip means it is selected.",
        "Or press \"+ Site or app\": pick something you use a lot, an app on this computer, or type any website. It is added as its own small list.",
        "Pick how long: 25 minutes, 1, 2 or 4 hours, \"…\" for any number of minutes, or \"Until…\" to choose the exact end (for example 7:00 tomorrow).",
        "Starts: \"Now\", or \"Later…\" to plan it (for example from 22:00 tonight). A planned lock is binding from the moment you set it.",
        "Choose how the session can be stopped (see the (i) next to that option).",
        "Press Start. The sites and apps in those lists are blocked in every browser and on the whole computer until the time is up."],
      note: "Each chosen list gets its own session, so you can stop or extend them separately under \"Active sessions\".",
    },
    lock_type: {
      t: "How a session can be stopped",
      p: ["This decides how hard it is to end the session early. A stronger lock helps you keep your promise to yourself in a weak moment."],
      how: ["Can be stopped any time: a Stop button ends it at once.",
        "Can be stopped only during set hours: Stop works only inside the daily window you choose (for example 20:00-21:00). The rest of the time it holds.",
        "Can't be stopped until the time is up: there is no way to end it early. It ends by itself.",
        "Typing a long text: to stop, you must type about 200 random characters exactly. Copy and paste is blocked. The pause gives you time to rethink.",
        "Restart the computer: the session can only be stopped after the computer has been restarted.",
        "Partner approval (if set up): your partner must approve before it ends."],
      note: "You will be asked to confirm before a locked session starts. Choose a length you can really keep.",
    },
    active_sessions: {
      t: "Active sessions",
      p: ["Every focus session that is running now, with the time left and its lock."],
      how: ["+15 min adds 15 minutes to that session. Extending is always allowed.",
        "Stop ends it, following its lock rule (for example typing the text, or waiting for a restart)."],
    },
    today_stats: {
      t: "Today and this week",
      p: ["\"Blocked attempts\" counts how many times something blocked was opened today. Each one is a moment the app protected you.",
        "\"Most time spent\" lists the sites and apps you used the most today. The bars show your total time for each day of this week."],
      note: "These numbers are stored only on this computer and never sent anywhere.",
    },
    lists_overview: {
      t: "Block lists",
      p: ["A block list is a group of things to block: websites, apps and words. Lists do nothing on their own until something switches them on.",
        "A list is switched on in one of four ways: \"Block all the time\" (24/7), a focus session from Home, a schedule, or a daily limit."],
      how: ["\"Add a ready-made list\" adds a list we keep up to date (social media, games, adult content…).",
        "\"New list\" lets you build your own.",
        "Edit changes a list. Delete removes it, along with its schedules and limits."],
      note: "When protection is on, lists that are in use are guarded: you can add to them but not remove items. This stops you from undoing your blocks in a weak moment.",
    },
    presets: {
      t: "Ready-made lists",
      p: ["Ready-made lists already contain the well-known sites and apps for a topic, so you don't need to type them yourself. We keep them up to date: new sites are added automatically through the internet.",
        "The numbers under each list show how many sites and apps it has. Adding the same list again merges the new items into the one you already have instead of making a copy."],
      note: "Adult content, gambling and dating lists are never shown item by item. Reading them could itself be a trigger.",
    },
    list_sites: {
      t: "Websites",
      p: ["Write one website per line. Blocking a site also blocks all of its subdomains, so facebook.com also blocks m.facebook.com and www.facebook.com."],
      how: ["facebook.com blocks the whole site.",
        "*.reddit.com does the same for every subdomain (written out for clarity).",
        "youtube.com/shorts blocks only that part of the site; the rest of YouTube keeps working. Blocking part of a site needs the browser extension.",
        "Don't include http:// or www. They are removed automatically."],
      note: "Whole sites are blocked for the entire computer (every browser and app), even without the extension.",
    },
    list_apps: {
      t: "Apps",
      p: ["Write one program per line. While the list is active, the program is closed within a few seconds whenever it is opened."],
      how: ["Use the program's name or its file name: Discord, telegram, steam.exe.",
        "To find the file name: open Task Manager (Ctrl+Shift+Esc), right-click the app → Go to details. The highlighted name (for example steam.exe) is the one to use."],
      note: "Windows system programs and HTL Content Guard itself can never be blocked, so your computer always keeps working.",
    },
    list_keywords: {
      t: "Words",
      p: ["Any web address (URL) that contains one of these words is blocked, on any site. This catches pages that aren't on any list, including search results.",
        "For example, the word \"casino\" blocks www.best-casino.net and google.com/search?q=casino."],
      note: "Word blocking works inside browsers that have the extension. Use full words. A very short word (like \"ad\") would block many normal pages.",
    },
    allowlist: {
      t: "Allowlist mode",
      p: ["Turns the list around: only the websites in this list are allowed, and every other website is blocked while the list is active.",
        "Useful for exam time or for children: for example allow only your school site, Wikipedia and Khan Academy."],
      note: "Search engines and other sites you still need must be added to the list, or they will be blocked too.",
    },
    always_on: {
      t: "Block all the time (24/7)",
      p: ["The list stays blocked every minute of every day, with no session or schedule needed. This is the right choice for content you never want to see, like adult sites or gambling.",
        "While protection is on, an always-on list can't be switched off or emptied. You can only add to it."],
      note: "Always-on lists are not offered in Focus sessions, Schedules or Limits because they are already blocked all the time.",
    },
    allow_exceptions: {
      t: "Always allowed (exceptions)",
      p: ["Some things in a blocked list are still useful: one YouTube channel for study, Gmail inside Google, or a game you allow. Write them here and this list leaves them open while it blocks everything else.",
        "An exception only lifts this list's own block. If another list that is active at the same time blocks the same site, it stays blocked."],
      how: ["A whole site: mail.google.com",
        "One page or channel: youtube.com/@khanacademy (pages work in browsers with the extension; whole sites work everywhere)",
        "An app: minecraft.exe",
        "One per line. http://, www. and the trailing / are removed for you."],
      note: "Adding an exception makes a list weaker. So while the list is locked or protection is on, a new exception is not added at once: it becomes a request and is added after the same waiting time as any other loosening (Protection → Requests). Removing an exception always works at once.",
    },
    limit_days: {
      t: "Different time on some days",
      p: ["Give a different allowance for each day of the week, for example 30 minutes on school days and 2 hours on the weekend. 0 means the list is blocked all that day."],
      how: ["Tick \"Different time on some days\" and change the days you want. The other days keep the daily minutes.",
        "Five minutes before today's time runs out, a small message tells you, so you can save your work or finish the video."],
      note: "While the limit is locked or protection is on, a day's time can only be lowered, not raised.",
    },
    week_view: {
      t: "Your week",
      p: ["Every active schedule drawn on one week, so you can see at a glance when each list is blocked. Each list has its own color; the line on today's row is now.",
        "A schedule that goes past midnight (for example bedtime 23:00-06:00) continues on the next day's row."],
      how: ["Hold the mouse over a colored bar to see the list and its times.",
        "To start fast, press \"New schedule\" and pick a ready choice: Work hours, School hours, Study evening, Bedtime or All day. You can still change the days and times."],
    },
    hidden_list: {
      t: "Why this list is hidden",
      p: ["For sensitive topics, seeing the list of sites can itself pull you back. So the app shows only how many items it contains, never the items themselves.",
        "You can still add more sites or words below. They are added to the list, which stays hidden."],
    },
    schedules_overview: {
      t: "Schedules",
      p: ["A schedule switches a list on automatically on the days and times you choose, every week, without you having to remember. For example: social media blocked Monday to Friday, 9:00 to 17:00, or every night from 23:00 to 7:00."],
      how: ["Press \"New schedule\" and choose a list, or press \"+ Site or app\" to pick one website or app.",
        "Tap the days it should run.",
        "Set the start and end times, then Save.",
        "The Enabled switch pauses or resumes a schedule without deleting it."],
      note: "When protection is on, an enabled schedule is guarded. It can be made longer, but not shorter or switched off.",
    },
    schedule_times: {
      t: "Start and end time",
      p: ["Blocking starts at the start time and stops at the end time on each chosen day.",
        "If the end time is earlier than the start time, the block runs overnight. For example 23:00 → 07:00 starts on the chosen day and ends the next morning.",
        "If start and end are the same, the list is blocked for the whole day."],
    },
    schedule_lock: {
      t: "Lock this schedule",
      p: ["A locked schedule can only be made stricter later: more days, or a longer time. It can't be shortened, paused or deleted.",
        "Lock it when you are sure about the routine and don't want to bargain with yourself later."],
    },
    limits_overview: {
      t: "Daily limits",
      p: ["A daily limit lets you use something for a set amount of time each day, then blocks it until midnight. Good for things you need a little of, but tend to overuse, like YouTube or games.",
        "The bar shows how much of today's allowance you have used. It resets at midnight."],
      how: ["Press \"New limit\" and choose a list, or press \"+ Site or app\" to limit just one website or app (for example YouTube).",
        "Set the maximum minutes per day.",
        "Optionally set how many times per day it may be opened."],
      note: "Time for websites is counted only while that site's tab is open and in focus. Time for apps is counted while the app is running.",
    },
    limit_minutes: {
      t: "Maximum minutes per day",
      p: ["How many minutes the list may be used in total each day. When the time runs out, everything in the list is blocked until midnight.",
        "0 means it is blocked all day.",
        "Five minutes before the time runs out, a small message on the screen tells you."],
    },
    limit_opens: {
      t: "Maximum opens per day",
      p: ["Optional. Limits how many separate times per day you can open something from the list, even if time is left. For example 3 opens stops the habit of checking every few minutes.",
        "Leave it empty for no limit on opens."],
    },
    limit_lock: {
      t: "Lock this limit",
      p: ["A locked limit can only be made stricter later: fewer minutes or fewer opens. It can't be raised, paused or deleted."],
    },
    protection_levels: {
      t: "Protection levels",
      p: ["Protection decides how hard it is to uninstall the app or loosen your blocks. It protects you from your own weak moments, not from other people.",
        "Off: everything can be changed or uninstalled at any time.",
        "Mindful: to uninstall or lower protection you must type a long text exactly and then wait 1 minute. That short pause is often enough to change your mind.",
        "Committed: uninstalling or lowering protection only happens after the waiting time you choose below (for example 3 days). This is the most effective level."],
      note: "Making protection stronger always happens at once. Making it weaker becomes a request that follows the current level's rule. Every level keeps a safe way out. Nobody is locked in forever.",
    },
    protection_delay: {
      t: "Waiting time (Committed level)",
      p: ["How long a request to uninstall or lower protection must wait before it is allowed. Longer waiting is stronger: the urge usually passes long before the time is up.",
        "A longer waiting time applies immediately. A shorter one needs a request, and that request has to wait for the current, longer time."],
      note: "\"Unlimited\" means the request never becomes ready by waiting. Choose it only if you are completely sure.",
    },
    uninstall: {
      t: "Uninstalling the app",
      p: ["When protection is on, the app can't simply be removed. Windows will refuse to stop it, and it restores itself if it is closed by force."],
      how: ["Optionally write why you want to uninstall. Reading your reason later can help.",
        "Press \"Request uninstall\".",
        "Complete the current level's rule: type the text (Mindful) or wait for the chosen time (Committed).",
        "When the request shows Approved, go to Windows Settings → Apps → HTL Content Guard PC → Uninstall within 24 hours."],
      note: "You can cancel a request at any time under \"Requests\".",
    },
    requests: {
      t: "Requests",
      p: ["Every request to uninstall, lower protection or end a locked session is listed here with its status."],
      how: ["Text not typed yet: press \"Type the text\" to complete it.",
        "Pending: waiting. \"Auto-approves at\" shows when it becomes ready.",
        "Approved: press Apply (or, for uninstall, uninstall from Windows Settings).",
        "Cancel removes a request you no longer want."],
    },
    premium_status: {
      t: "Trial and Premium",
      p: ["Every computer gets a free 4-day trial with all features. After the trial, everything you already set up keeps blocking exactly as before. Premium is only needed to add new blocks or make them stronger.",
        "Premium unlocks all features for the period of your plan (or forever with Lifetime)."],
      note: "If the app can't reach the internet for a long time, Premium is shown as \"Offline\". Connect and press \"Refresh status\".",
    },
    premium_buy: {
      t: "Buying Premium with bKash or Nagad",
      how: ["Choose a plan. Yearly gives the best value.",
        "Choose bKash or Nagad, then in that app use Send Money to send the exact amount to the number shown. The Copy button copies the number.",
        "After paying you receive a message with a TrxID (transaction ID). Type it in step 3 and press Submit.",
        "Done. We check the payment and Premium turns on by itself, usually within a few hours. You don't need to keep the app open."],
      note: "Use Send Money, not Cash Out or Payment. Each TrxID can be used only once.",
    },
    trx: {
      t: "TrxID (transaction ID)",
      p: ["The TrxID is the code in the confirmation message bKash or Nagad sends you after a payment, for example 9FG7ABC123. It proves your payment, so type it exactly: letters and numbers only."],
      note: "Your phone number is optional. We only use it to contact you if there is a problem with the payment.",
    },
    activation_code: {
      t: "Activation code",
      p: ["If you received a code (for example PREMIUM-XXXX) from us or from a reseller, enter it here to turn on Premium at once. Each code works only once."],
    },
    device_id: {
      t: "Device ID",
      p: ["A unique code for this computer. Your trial and Premium are linked to it. It doesn't contain your name or any personal information.",
        "Copy it and include it whenever you contact support, so we can find your purchase quickly."],
    },
    language: {
      t: "Language",
      p: ["Changes the language of the whole app and of the block page in your browser. Press Save after choosing."],
    },
    goal: {
      t: "My goal",
      p: ["A short sentence about why you are doing this, for example \"Finish my exam syllabus this month\". It appears on the block page every time something is blocked, so you are reminded of what matters at the exact moment you need it."],
    },
    unsupported_browsers: {
      t: "Close browsers that can't be filtered",
      p: ["Some browsers can't be filtered at all. Tor Browser, for example, looks up websites through its own network, so blocked sites would open in it. While anything is being blocked, these browsers are closed a few seconds after they start.",
        "This doesn't affect normal browsers like Chrome, Edge, Firefox, Brave or Opera. It is on by default and recommended for everyone."],
      note: "Can't be switched off while protection is on. See \"Which browsers are supported\" for the full list.",
    },
    guard_browsers: {
      t: "Close browsers where the extension isn't working",
      p: ["Blocking parts of sites (Shorts, Reels) and words happens inside the browser through the extension. Without this switch, someone could get around that by removing the extension or switching to a browser that doesn't have it.",
        "When it is on, while anything is being blocked, a supported browser whose extension isn't installed or isn't working is closed after a 90-second warning. It also installs the extension automatically in every browser where that's possible.",
        "Even with this switch off, Chrome, Brave, Vivaldi and Chromium are always guarded: the extension installs itself there, so if it is missing someone removed or turned it off. And while a list has a single-page exception (like one YouTube channel), every browser without the extension is closed, because only the extension can keep the rest of that site blocked."],
      how: ["First turn on \"Install the extension permanently\" and check \"Browsers running now\": every browser you use should show Protected.",
        "Then turn this on. A browser that still shows \"Not filtered – will be closed\" will be closed while blocking."],
      note: "Can't be switched off while protection is on.",
    },
    running_browsers: {
      t: "Browsers running now",
      p: ["A live check of every browser open on this computer.",
        "Protected: the extension is working. Not filtered – will be closed: it will be closed while blocking. Not filtered: no extension, but browser protection is off."],
    },
    safe_search: {
      t: "Force SafeSearch",
      p: ["Makes Google, Bing, DuckDuckGo and YouTube always use their strictest safe mode, so explicit images, videos and results are hidden from search. It works in every browser and app on this computer, even without the extension, and can't be turned off from the search page."],
    },
    private_windows: {
      t: "Block private (incognito) windows",
      p: ["Disables Incognito / InPrivate windows in Chrome, Edge, Brave, Vivaldi and Firefox. Private windows are often where extensions don't run, so blocking them closes that gap.",
        "On by default for new installs. While protection is on, the browsers' own \"secure DNS\" is also turned off, so every site name passes the block list (normal browsing is not affected)."],
      note: "Can't be switched off while protection is on.",
    },
    managed_ext: {
      t: "Install the extension permanently",
      p: ["Uses the official browser policy that schools and companies use. The extension installs itself in Chrome, Edge, Brave and Firefox, can't be removed or disabled, and comes back even if the browser is reinstalled."],
      note: "Turned on automatically together with browser protection. Can't be switched off while protection is on.",
    },
    browser_lists: {
      t: "Supported browsers",
      p: ["Supported browsers can run the extension, so every feature works in them: partial blocking, words and SafeSearch.",
        "Unsupported browsers can't be filtered from the inside. Whole sites are still blocked in them, and with browser protection on they are closed while blocking."],
    },
    filters: {
      t: "Partial blocking",
      p: ["Hides only the most addictive part of a site, while the rest keeps working. For example, you can still watch a YouTube video you searched for, but Shorts and the endless suggestions are gone.",
        "Tick the parts you want hidden and press Save. It applies all the time in every browser that has the extension."],
    },
    backup: {
      t: "Backup and restore",
      p: ["All your lists, schedules, limits and settings as text. Copy it and keep it somewhere safe, for example in an email to yourself.",
        "On a new computer, press Import and paste it to restore everything. Protection rules still apply: an import can't loosen locked items."],
    },
    updates: {
      t: "Updates",
      p: ["The app updates itself automatically when a new version is released. Every update is checked with our digital signature before it is installed, so a fake update can't be installed.",
        "Updating keeps all your lists, schedules, protection and Premium. \"Update now\" installs a waiting update at once."],
    },
    extension: {
      t: "Browser extension",
      p: ["The HTL Content Guard PC extension filters inside the browser: partial blocking (Shorts, Reels, feeds), word blocking, the block page and SafeSearch.",
        "It installs itself in Chrome, Edge and Brave when browser protection is on. You can also add it yourself from the Chrome Web Store by searching \"HTL Content Guard PC\"."],
    },
  },

  bn: {
    home_status: {
      t: "ব্লকিং অবস্থা",
      p: ["এই কার্ডে দেখায় এই মুহূর্তে কী ব্লক হচ্ছে আর কেন। সবুজ মানে অন্তত একটা লিস্ট চালু আছে; ধূসর মানে এখন কিছুই ব্লক নেই।",
        "প্রতিটা লাইনে থাকে লিস্টের নাম, কারণ (সবসময় ব্লক, ফোকাস সেশন, সময়সূচি বা দৈনিক সীমা) আর কখন শেষ হবে। 🔒 মানে লক করা, আগে বন্ধ করা যাবে না।"],
      note: "\"সবসময় ব্লক\" লিস্ট (যেমন অ্যাডাল্ট কনটেন্ট) ২৪ ঘণ্টা ব্লক থাকে, কখনো শেষ হয় না।",
    },
    quick_start: {
      t: "ফোকাস সেশন",
      p: ["ফোকাস সেশন আপনার বেছে নেওয়া লিস্টগুলো এখন থেকে নির্দিষ্ট সময় পর্যন্ত ব্লক রাখে। পড়া, কাজ বা ঘুমের সময় মনোযোগ ধরে রাখতে এটা ব্যবহার করুন।"],
      how: ["এক বা একাধিক লিস্ট চাপুন (যেমন সোশ্যাল মিডিয়া)। রঙ ভরা থাকলে বেছে নেওয়া হয়েছে।",
        "অথবা \"+ সাইট বা অ্যাপ\" চাপুন: আপনি বেশি ব্যবহার করেন এমন কিছু, এই কম্পিউটারের কোনো অ্যাপ, বা যেকোনো ওয়েবসাইট লিখে বেছে নিন। এটা নিজের একটা ছোট লিস্ট হিসেবে যোগ হয়।",
        "কতক্ষণ বেছে নিন: ২৫ মিনিট, ১, ২ বা ৪ ঘণ্টা, \"…\" দিয়ে যেকোনো মিনিট, অথবা \"কখন পর্যন্ত…\" দিয়ে ঠিক শেষের সময় (যেমন কাল সকাল ৭টা)।",
        "শুরু: \"এখনই\", অথবা \"পরে…\" দিয়ে আগে থেকে সেট করুন (যেমন আজ রাত ১০টা থেকে)। আগে থেকে সেট করা লক সেট করার মুহূর্ত থেকেই বাঁধাধরা।",
        "সেশন কীভাবে থামানো যাবে তা বেছে নিন (ওই অপশনের পাশের (i) দেখুন)।",
        "শুরু চাপুন। সময় শেষ না হওয়া পর্যন্ত ওই লিস্টের সাইট আর অ্যাপ সব ব্রাউজারে ও পুরো কম্পিউটারে ব্লক থাকবে।"],
      note: "প্রতিটা লিস্টের জন্য আলাদা সেশন হয়, তাই \"চলমান সেশন\"-এ আলাদা করে থামানো বা বাড়ানো যায়।",
    },
    lock_type: {
      t: "সেশন কীভাবে থামানো যাবে",
      p: ["এটা ঠিক করে সেশন আগে থামানো কতটা কঠিন হবে। শক্ত লক দুর্বল মুহূর্তে নিজের কাছে দেওয়া কথা রাখতে সাহায্য করে।"],
      how: ["যেকোনো সময় থামানো যাবে: থামান চাপলেই শেষ।",
        "শুধু নির্দিষ্ট সময়ে থামানো যাবে: আপনার বেছে নেওয়া দৈনিক সময়ের মধ্যেই (যেমন রাত ৮টা–৯টা) থামান কাজ করে, বাকি সময় লক থাকে।",
        "সময় শেষ না হলে থামানো যাবে না: আগে শেষ করার কোনো উপায় নেই, নিজে থেকে শেষ হবে।",
        "লম্বা লেখা টাইপ করে: থামাতে প্রায় ২০০টা এলোমেলো অক্ষর হুবহু টাইপ করতে হবে। কপি-পেস্ট চলে না। এই বিরতিতে আবার ভাবার সুযোগ পাবেন।",
        "কম্পিউটার রিস্টার্ট: কম্পিউটার রিস্টার্ট করার পরেই সেশন থামানো যাবে।",
        "সঙ্গীর অনুমতি (চালু থাকলে): আপনার সঙ্গী অনুমতি দিলে তবেই শেষ হবে।"],
      note: "লক করা সেশন শুরুর আগে নিশ্চিত করতে বলা হবে। যতটা সময় সত্যিই রাখতে পারবেন ততটাই দিন।",
    },
    active_sessions: {
      t: "চলমান সেশন",
      p: ["এখন যত ফোকাস সেশন চলছে, বাকি সময় আর লক সহ।"],
      how: ["+১৫ মিনিট ওই সেশনে ১৫ মিনিট যোগ করে। বাড়ানো সবসময় যায়।",
        "থামান সেশনটা তার লকের নিয়ম মেনে শেষ করে (যেমন লেখা টাইপ করা বা রিস্টার্টের অপেক্ষা)।"],
    },
    today_stats: {
      t: "আজ ও এই সপ্তাহ",
      p: ["\"ব্লক করা চেষ্টা\" দেখায় আজ কতবার ব্লক করা কিছু খোলার চেষ্টা হয়েছে। প্রতিটা হলো একেকটা মুহূর্ত যখন অ্যাপ আপনাকে রক্ষা করেছে।",
        "\"সবচেয়ে বেশি সময়\" দেখায় আজ কোন সাইট/অ্যাপে সবচেয়ে বেশি সময় দিয়েছেন। বারগুলো এই সপ্তাহের প্রতিদিনের মোট সময়।"],
      note: "এই হিসাব শুধু এই কম্পিউটারেই থাকে, কোথাও পাঠানো হয় না।",
    },
    lists_overview: {
      t: "ব্লক লিস্ট",
      p: ["ব্লক লিস্ট হলো যা ব্লক করতে চান তার একটা দল: ওয়েবসাইট, অ্যাপ আর শব্দ। কিছু একটা চালু না করা পর্যন্ত লিস্ট নিজে কিছু করে না।",
        "লিস্ট চালু হয় চারভাবে: \"সবসময় ব্লক\" (২৪/৭), হোম থেকে ফোকাস সেশন, সময়সূচি, বা দৈনিক সীমা।"],
      how: ["\"তৈরি লিস্ট যোগ করুন\" দিয়ে আমাদের হালনাগাদ রাখা লিস্ট নিন (সোশ্যাল মিডিয়া, গেম, অ্যাডাল্ট…)।",
        "\"নতুন লিস্ট\" দিয়ে নিজের লিস্ট বানান।",
        "এডিট দিয়ে বদলান। মুছুন দিলে লিস্ট আর তার সময়সূচি ও সীমা মুছে যায়।"],
      note: "সুরক্ষা চালু থাকলে ব্যবহৃত লিস্ট সুরক্ষিত থাকে: যোগ করা যায়, কিন্তু সরানো যায় না। যাতে দুর্বল মুহূর্তে নিজের ব্লক নিজে ভেঙে না ফেলেন।",
    },
    presets: {
      t: "তৈরি লিস্ট",
      p: ["তৈরি লিস্টে একটা বিষয়ের পরিচিত সাইট আর অ্যাপ আগে থেকেই থাকে, নিজে টাইপ করতে হয় না। আমরা এগুলো হালনাগাদ রাখি: নতুন সাইট ইন্টারনেটের মাধ্যমে নিজে থেকে যোগ হয়।",
        "প্রতিটার নিচের সংখ্যা হলো কতগুলো সাইট ও অ্যাপ আছে। একই লিস্ট আবার যোগ করলে কপি না বানিয়ে নতুনগুলো আগেরটার সাথে মিলে যায়।"],
      note: "অ্যাডাল্ট, জুয়া আর ডেটিং লিস্টের ভেতরের নামগুলো কখনো দেখানো হয় না, কারণ সেগুলো পড়াই আসক্তি জাগাতে পারে।",
    },
    list_sites: {
      t: "ওয়েবসাইট",
      p: ["প্রতি লাইনে একটা ওয়েবসাইট লিখুন। কোনো সাইট ব্লক করলে তার সব সাব-ডোমেইনও ব্লক হয়, তাই facebook.com দিলে m.facebook.com আর www.facebook.com-ও ব্লক হয়।"],
      how: ["facebook.com: পুরো সাইট ব্লক।",
        "*.reddit.com: একই কাজ, সব সাব-ডোমেইন সহ (বোঝার সুবিধার জন্য লেখা)।",
        "youtube.com/shorts: শুধু ওই অংশ ব্লক, বাকি YouTube চলবে। সাইটের অংশ ব্লক করতে ব্রাউজার এক্সটেনশন লাগে।",
        "http:// বা www. লিখবেন না, নিজে থেকেই বাদ যায়।"],
      note: "পুরো সাইট পুরো কম্পিউটারে (সব ব্রাউজার আর অ্যাপে) ব্লক হয়, এক্সটেনশন ছাড়াও।",
    },
    list_apps: {
      t: "অ্যাপ",
      p: ["প্রতি লাইনে একটা প্রোগ্রাম লিখুন। লিস্ট চালু থাকলে প্রোগ্রামটা খুললেই কয়েক সেকেন্ডের মধ্যে বন্ধ হয়ে যায়।"],
      how: ["প্রোগ্রামের নাম বা ফাইলের নাম লিখুন: Discord, telegram, steam.exe।",
        "ফাইলের নাম বের করতে: Task Manager খুলুন (Ctrl+Shift+Esc), অ্যাপে রাইট-ক্লিক → Go to details। হাইলাইট করা নামটাই (যেমন steam.exe) লিখবেন।"],
      note: "Windows-এর সিস্টেম প্রোগ্রাম আর HTL Content Guard নিজে কখনো ব্লক হয় না, তাই কম্পিউটার সবসময় চলবে।",
    },
    list_keywords: {
      t: "শব্দ",
      p: ["যেকোনো ওয়েব ঠিকানায় (URL) এই শব্দগুলোর কোনোটা থাকলে সেটা ব্লক হয়, যেকোনো সাইটে। এতে কোনো লিস্টে না থাকা পাতা, এমনকি সার্চের ফলাফলও ধরা পড়ে।",
        "যেমন \"casino\" শব্দ দিলে www.best-casino.net আর google.com/search?q=casino দুটোই ব্লক হয়।"],
      note: "শব্দ ব্লক এক্সটেনশন থাকা ব্রাউজারে কাজ করে। পুরো শব্দ দিন। খুব ছোট শব্দ (যেমন \"ad\") অনেক সাধারণ পাতাও ব্লক করে দেবে।",
    },
    allowlist: {
      t: "শুধু-অনুমোদিত মোড",
      p: ["লিস্টটা উল্টে দেয়: লিস্ট চালু থাকার সময় শুধু এই লিস্টের সাইটগুলো খোলা যাবে, বাকি সব ওয়েবসাইট ব্লক।",
        "পরীক্ষার সময় বা শিশুদের জন্য কাজের: যেমন শুধু স্কুলের সাইট, Wikipedia আর Khan Academy খোলা রাখুন।"],
      note: "সার্চ ইঞ্জিন বা দরকারি অন্য সাইটও লিস্টে যোগ করতে হবে, নইলে সেগুলোও ব্লক হবে।",
    },
    always_on: {
      t: "সবসময় ব্লক (২৪/৭)",
      p: ["লিস্টটা প্রতিদিন প্রতি মিনিট ব্লক থাকে, কোনো সেশন বা সময়সূচি লাগে না। যা কখনো দেখতে চান না (যেমন অ্যাডাল্ট সাইট বা জুয়া), তার জন্য এটাই ঠিক।",
        "সুরক্ষা চালু থাকলে সবসময়-ব্লক লিস্ট বন্ধ বা খালি করা যায় না, শুধু যোগ করা যায়।"],
      note: "এই লিস্ট ফোকাস সেশন, সময়সূচি বা সীমায় দেখানো হয় না, কারণ এগুলো এমনিতেই সবসময় ব্লক।",
    },
    allow_exceptions: {
      t: "সবসময় অনুমোদিত (ব্যতিক্রম)",
      p: ["ব্লক লিস্টের কিছু জিনিস তবুও কাজের হতে পারে: পড়াশোনার একটা YouTube চ্যানেল, Google-এর মধ্যে Gmail, বা অনুমতি দেওয়া কোনো গেম। এখানে লিখুন — লিস্টটা বাকি সব ব্লক করলেও এগুলো খোলা রাখবে।",
        "ব্যতিক্রম শুধু এই লিস্টের নিজের ব্লক তোলে। একই সময়ে চালু অন্য কোনো লিস্ট একই সাইট ব্লক করলে সেটা ব্লকই থাকবে।"],
      how: ["পুরো সাইট: mail.google.com",
        "একটা পাতা বা চ্যানেল: youtube.com/@khanacademy (পাতা কাজ করে এক্সটেনশন থাকা ব্রাউজারে; পুরো সাইট সবখানে)",
        "অ্যাপ: minecraft.exe",
        "প্রতি লাইনে একটা। http://, www. আর শেষের / নিজে থেকে বাদ যায়।"],
      note: "ব্যতিক্রম যোগ করলে লিস্ট দুর্বল হয়। তাই লিস্ট লক বা সুরক্ষা চালু থাকলে নতুন ব্যতিক্রম সাথে সাথে যোগ হয় না: এটা একটা অনুরোধ হয়ে যায়, আর অন্য যেকোনো ঢিলা করার মতো একই অপেক্ষার সময় শেষে যোগ হয় (সুরক্ষা → অনুরোধ)। ব্যতিক্রম সরানো সবসময় সাথে সাথে হয়।",
    },
    limit_days: {
      t: "কিছু দিনে আলাদা সময়",
      p: ["সপ্তাহের প্রতিটা দিনের জন্য আলাদা সময় দিন, যেমন স্কুলের দিনে ৩০ মিনিট আর ছুটির দিনে ২ ঘণ্টা। ০ মানে সেদিন সারাদিন লিস্টটা ব্লক।"],
      how: ["\"কিছু দিনে আলাদা সময়\" টিক দিয়ে যে দিনগুলো চান বদলান। বাকি দিনে দৈনিক মিনিটই থাকবে।",
        "আজকের সময় শেষ হওয়ার ৫ মিনিট আগে একটা ছোট বার্তা জানিয়ে দেবে, যাতে কাজ সেভ করতে বা ভিডিও শেষ করতে পারেন।"],
      note: "সীমা লক থাকলে বা সুরক্ষা চালু থাকলে কোনো দিনের সময় শুধু কমানো যায়, বাড়ানো যায় না।",
    },
    week_view: {
      t: "আপনার সপ্তাহ",
      p: ["চালু থাকা সব সময়সূচি এক সপ্তাহের ছবিতে, যাতে এক নজরে বোঝা যায় কোন লিস্ট কখন ব্লক। প্রতিটা লিস্টের আলাদা রং; আজকের সারির দাগটা এখনকার সময়।",
        "যে সময়সূচি মাঝরাত পেরোয় (যেমন ঘুমের সময় রাত ১১টা–সকাল ৬টা), সেটা পরের দিনের সারিতে চলতে থাকে।"],
      how: ["রঙিন দাগের ওপর মাউস রাখলে লিস্ট আর সময় দেখাবে।",
        "দ্রুত শুরু করতে \"নতুন সময়সূচি\" চেপে একটা তৈরি অপশন বাছুন: অফিস সময়, স্কুল সময়, সন্ধ্যার পড়া, ঘুমের সময় বা সারাদিন। পরে দিন আর সময় বদলানো যায়।"],
    },
    hidden_list: {
      t: "লিস্টটা কেন লুকানো",
      p: ["স্পর্শকাতর বিষয়ে সাইটের নাম দেখাটাই আবার টেনে নিতে পারে। তাই অ্যাপ শুধু কতগুলো আছে তা দেখায়, নামগুলো কখনো দেখায় না।",
        "নিচে আরও সাইট বা শব্দ যোগ করতে পারবেন। সেগুলো লিস্টে যোগ হবে, লিস্টটা লুকানোই থাকবে।"],
    },
    schedules_overview: {
      t: "সময়সূচি",
      p: ["সময়সূচি আপনার বেছে নেওয়া দিন আর সময়ে প্রতি সপ্তাহে নিজে থেকে লিস্ট চালু করে, মনে রাখতে হয় না। যেমন: সোমবার থেকে শুক্রবার সকাল ৯টা থেকে বিকেল ৫টা সোশ্যাল মিডিয়া বন্ধ, বা প্রতি রাতে ১১টা থেকে সকাল ৭টা।"],
      how: ["\"নতুন সময়সূচি\" চেপে লিস্ট বেছে নিন, অথবা \"+ সাইট বা অ্যাপ\" চেপে একটা ওয়েবসাইট বা অ্যাপ বেছে নিন।",
        "যেসব দিনে চলবে সেগুলো চাপুন।",
        "শুরু আর শেষের সময় দিয়ে সেভ করুন।",
        "চালু সুইচ দিয়ে না মুছে বিরতি দেওয়া বা আবার চালু করা যায়।"],
      note: "সুরক্ষা চালু থাকলে চালু সময়সূচি সুরক্ষিত থাকে: লম্বা করা যায়, কিন্তু ছোট বা বন্ধ করা যায় না।",
    },
    schedule_times: {
      t: "শুরু আর শেষের সময়",
      p: ["প্রতিটা বেছে নেওয়া দিনে শুরুর সময়ে ব্লক শুরু হয়, শেষের সময়ে থামে।",
        "শেষের সময় শুরুর চেয়ে আগে হলে ব্লক রাত পেরিয়ে চলে। যেমন রাত ১১টা → সকাল ৭টা বেছে নেওয়া দিনে শুরু হয়ে পরদিন সকালে শেষ হয়।",
        "শুরু আর শেষ একই হলে পুরো দিন ব্লক থাকে।"],
    },
    schedule_lock: {
      t: "সময়সূচি লক",
      p: ["লক করা সময়সূচি পরে শুধু আরও কড়া করা যায়: বেশি দিন বা বেশি সময়। ছোট করা, বিরতি দেওয়া বা মোছা যায় না।",
        "রুটিন নিয়ে নিশ্চিত হলে আর পরে নিজের সাথে দর-কষাকষি করতে না চাইলে লক করুন।"],
    },
    limits_overview: {
      t: "দৈনিক সীমা",
      p: ["দৈনিক সীমা দিয়ে কোনো কিছু প্রতিদিন নির্দিষ্ট সময় ব্যবহার করা যায়, তারপর মাঝরাত পর্যন্ত ব্লক। যা একটু দরকার কিন্তু বেশি হয়ে যায় (যেমন YouTube বা গেম), তার জন্য ভালো।",
        "বারে দেখায় আজকের বরাদ্দের কতটা ব্যবহার হয়েছে। মাঝরাতে আবার শুরু হয়।"],
      how: ["\"নতুন সীমা\" চেপে লিস্ট বেছে নিন, অথবা \"+ সাইট বা অ্যাপ\" চেপে শুধু একটা ওয়েবসাইট বা অ্যাপের সীমা দিন (যেমন YouTube)।",
        "দিনে সর্বোচ্চ কত মিনিট তা দিন।",
        "চাইলে দিনে কতবার খোলা যাবে তাও দিন।"],
      note: "ওয়েবসাইটের সময় গোনা হয় শুধু যখন ওই সাইটের ট্যাব সামনে খোলা থাকে। অ্যাপের সময় গোনা হয় অ্যাপ চালু থাকা পর্যন্ত।",
    },
    limit_minutes: {
      t: "দিনে সর্বোচ্চ মিনিট",
      p: ["প্রতিদিন লিস্টটা মোট কত মিনিট ব্যবহার করা যাবে। সময় শেষ হলে লিস্টের সবকিছু মাঝরাত পর্যন্ত ব্লক।",
        "০ মানে সারাদিন ব্লক।",
        "সময় শেষ হওয়ার ৫ মিনিট আগে স্ক্রিনে একটা ছোট বার্তা জানিয়ে দেয়।"],
    },
    limit_opens: {
      t: "দিনে সর্বোচ্চ কতবার খোলা",
      p: ["ঐচ্ছিক। সময় বাকি থাকলেও দিনে কতবার আলাদাভাবে খোলা যাবে তা সীমিত করে। যেমন ৩ বার দিলে কয়েক মিনিট পরপর দেখার অভ্যাস থামে।",
        "খালি রাখলে খোলার কোনো সীমা নেই।"],
    },
    limit_lock: {
      t: "সীমা লক",
      p: ["লক করা সীমা পরে শুধু আরও কড়া করা যায়: কম মিনিট বা কমবার খোলা। বাড়ানো, বিরতি দেওয়া বা মোছা যায় না।"],
    },
    protection_levels: {
      t: "সুরক্ষার স্তর",
      p: ["সুরক্ষা ঠিক করে অ্যাপ আনইনস্টল বা ব্লক ঢিলা করা কতটা কঠিন। এটা আপনাকে আপনার নিজের দুর্বল মুহূর্ত থেকে রক্ষা করে, অন্য মানুষ থেকে নয়।",
        "বন্ধ: যেকোনো সময় সব বদলানো বা আনইনস্টল করা যায়।",
        "সচেতন: আনইনস্টল বা সুরক্ষা কমাতে একটা লম্বা লেখা হুবহু টাইপ করে ১ মিনিট অপেক্ষা করতে হয়। এই ছোট বিরতিতেই অনেক সময় মন বদলে যায়।",
        "প্রতিশ্রুতিবদ্ধ: নিচে বেছে নেওয়া অপেক্ষার সময় (যেমন ৩ দিন) পার হওয়ার পরেই আনইনস্টল বা সুরক্ষা কমানো যায়। এটাই সবচেয়ে কার্যকর।"],
      note: "সুরক্ষা বাড়ানো সবসময় সাথে সাথে হয়। কমানো একটা অনুরোধ হয়ে যায়, যা বর্তমান স্তরের নিয়ম মেনে চলে। প্রতিটা স্তরে বের হওয়ার নিরাপদ পথ আছে, কেউ চিরকাল আটকে থাকে না।",
    },
    protection_delay: {
      t: "অপেক্ষার সময় (প্রতিশ্রুতিবদ্ধ স্তর)",
      p: ["আনইনস্টল বা সুরক্ষা কমানোর অনুরোধ অনুমতি পাওয়ার আগে কতক্ষণ অপেক্ষা করবে। যত লম্বা তত শক্ত: সময় শেষ হওয়ার অনেক আগেই ইচ্ছাটা সাধারণত চলে যায়।",
        "লম্বা সময় সাথে সাথে চালু হয়। ছোট করতে অনুরোধ লাগে, আর সেই অনুরোধকে বর্তমানের লম্বা সময়টাই অপেক্ষা করতে হয়।"],
      note: "\"সীমাহীন\" মানে অপেক্ষা করে অনুরোধ কখনো তৈরি হবে না। পুরোপুরি নিশ্চিত হলে তবেই বেছে নিন।",
    },
    uninstall: {
      t: "অ্যাপ আনইনস্টল",
      p: ["সুরক্ষা চালু থাকলে অ্যাপ সরাসরি মোছা যায় না। Windows এটা থামাতে দেয় না, আর জোর করে বন্ধ করলে নিজে আবার চালু হয়।"],
      how: ["চাইলে কেন আনইনস্টল করতে চান লিখুন। পরে নিজের কারণ পড়লে কাজে লাগতে পারে।",
        "\"আনইনস্টলের অনুরোধ করুন\" চাপুন।",
        "বর্তমান স্তরের নিয়ম পূরণ করুন: লেখা টাইপ (সচেতন) বা বেছে নেওয়া সময় অপেক্ষা (প্রতিশ্রুতিবদ্ধ)।",
        "অনুরোধে অনুমোদিত দেখালে ২৪ ঘণ্টার মধ্যে Windows Settings → Apps → HTL Content Guard PC → Uninstall করুন।"],
      note: "\"অনুরোধ\"-এর নিচে যেকোনো সময় অনুরোধ বাতিল করা যায়।",
    },
    requests: {
      t: "অনুরোধ",
      p: ["আনইনস্টল, সুরক্ষা কমানো বা লক করা সেশন শেষ করার প্রতিটা অনুরোধ এখানে অবস্থাসহ দেখায়।"],
      how: ["লেখা টাইপ হয়নি: \"লেখা টাইপ করুন\" চেপে শেষ করুন।",
        "অপেক্ষমাণ: অপেক্ষা চলছে। \"নিজে অনুমোদন হবে\" দেখায় কখন তৈরি হবে।",
        "অনুমোদিত: প্রয়োগ চাপুন (আনইনস্টলের ক্ষেত্রে Windows Settings থেকে আনইনস্টল করুন)।",
        "বাতিল দিলে যে অনুরোধ আর চান না সেটা মুছে যায়।"],
    },
    premium_status: {
      t: "ট্রায়াল আর Premium",
      p: ["প্রতিটা কম্পিউটার সব ফিচার সহ ৪ দিনের ফ্রি ট্রায়াল পায়। ট্রায়ালের পরেও যা সেট করেছেন সব আগের মতোই ব্লক করতে থাকে। শুধু নতুন ব্লক যোগ বা আরও শক্ত করতে Premium লাগে।",
        "Premium প্ল্যানের মেয়াদ পর্যন্ত (Lifetime হলে চিরকাল) সব ফিচার খুলে দেয়।"],
      note: "অনেকদিন ইন্টারনেট না পেলে Premium \"অফলাইন\" দেখায়। ইন্টারনেট চালু করে \"অবস্থা রিফ্রেশ\" চাপুন।",
    },
    premium_buy: {
      t: "বিকাশ বা নগদে Premium কেনা",
      how: ["প্ল্যান বেছে নিন। বার্ষিকে সবচেয়ে সাশ্রয়।",
        "বিকাশ বা নগদ বেছে নিন, তারপর সেই অ্যাপে Send Money দিয়ে দেখানো নম্বরে ঠিক ওই টাকা পাঠান। কপি বোতামে নম্বর কপি হয়।",
        "টাকা পাঠানোর পর মেসেজে একটা TrxID আসবে। সেটা ৩ নম্বর ধাপে লিখে জমা দিন চাপুন।",
        "ব্যস। আমরা যাচাই করলে Premium নিজে থেকে চালু হয়, সাধারণত কয়েক ঘণ্টার মধ্যে। অ্যাপ খোলা রাখতে হবে না।"],
      note: "Send Money ব্যবহার করুন, Cash Out বা Payment নয়। প্রতিটা TrxID একবারই ব্যবহার করা যায়।",
    },
    trx: {
      t: "TrxID (লেনদেন নম্বর)",
      p: ["TrxID হলো টাকা পাঠানোর পর বিকাশ/নগদের নিশ্চিতকরণ মেসেজে আসা কোড, যেমন 9FG7ABC123। এটা আপনার পেমেন্টের প্রমাণ, তাই হুবহু লিখুন: শুধু অক্ষর আর সংখ্যা।"],
      note: "মোবাইল নম্বর ঐচ্ছিক। পেমেন্টে সমস্যা হলে যোগাযোগের জন্যই শুধু ব্যবহার হয়।",
    },
    activation_code: {
      t: "অ্যাক্টিভেশন কোড",
      p: ["আমাদের বা কোনো বিক্রেতার কাছ থেকে কোড (যেমন PREMIUM-XXXX) পেলে এখানে লিখুন, সাথে সাথে Premium চালু হবে। প্রতিটা কোড একবারই চলে।"],
    },
    device_id: {
      t: "ডিভাইস আইডি",
      p: ["এই কম্পিউটারের একটা অনন্য কোড। আপনার ট্রায়াল আর Premium এর সাথে যুক্ত। এতে আপনার নাম বা কোনো ব্যক্তিগত তথ্য নেই।",
        "সাপোর্টে যোগাযোগের সময় এটা কপি করে দিন, তাহলে আপনার কেনাকাটা দ্রুত খুঁজে পাব।"],
    },
    language: {
      t: "ভাষা",
      p: ["পুরো অ্যাপ আর ব্রাউজারের ব্লক পাতার ভাষা বদলায়। বেছে নিয়ে সেভ চাপুন।"],
    },
    goal: {
      t: "আমার লক্ষ্য",
      p: ["কেন এটা করছেন তা নিয়ে একটা ছোট বাক্য, যেমন \"এই মাসে পরীক্ষার পুরো সিলেবাস শেষ করব\"। যখনই কিছু ব্লক হয়, ব্লক পাতায় এটা দেখায়, ঠিক যখন মনে করিয়ে দেওয়া সবচেয়ে দরকার।"],
    },
    unsupported_browsers: {
      t: "যে ব্রাউজার ফিল্টার করা যায় না তা বন্ধ করা",
      p: ["কিছু ব্রাউজার একেবারেই ফিল্টার করা যায় না। যেমন Tor Browser নিজের নেটওয়ার্ক দিয়ে ওয়েবসাইট খোঁজে, তাই ব্লক করা সাইট সেখানে খুলে যেত। কিছু ব্লক থাকা অবস্থায় এই ব্রাউজারগুলো চালু হওয়ার কয়েক সেকেন্ডের মধ্যে বন্ধ হয়ে যায়।",
        "Chrome, Edge, Firefox, Brave বা Opera-র মতো সাধারণ ব্রাউজারে এর কোনো প্রভাব নেই। এটা ডিফল্টে চালু থাকে, আর সবার জন্যই সুপারিশ করা হয়।"],
      note: "সুরক্ষা চালু থাকলে বন্ধ করা যায় না। পুরো তালিকা \"কোন ব্রাউজার সমর্থিত\"-এ দেখুন।",
    },
    guard_browsers: {
      t: "যে ব্রাউজারে এক্সটেনশন কাজ করছে না তা বন্ধ করা",
      p: ["সাইটের অংশ (Shorts, Reels) আর শব্দ ব্লক হয় ব্রাউজারের ভেতরে, এক্সটেনশন দিয়ে। এই সুইচ ছাড়া কেউ এক্সটেনশন সরিয়ে বা এক্সটেনশন ছাড়া অন্য ব্রাউজারে গিয়ে এটা এড়াতে পারত।",
        "চালু থাকলে কিছু ব্লক থাকা অবস্থায় যে সমর্থিত ব্রাউজারে এক্সটেনশন বসেনি বা কাজ করছে না, তা ৯০ সেকেন্ডের সতর্কতার পর বন্ধ হয়। যেখানে সম্ভব সেখানে এটা এক্সটেনশন নিজে থেকে বসিয়েও দেয়।",
        "এই সুইচ বন্ধ থাকলেও Chrome, Brave, Vivaldi আর Chromium সবসময় পাহারায় থাকে: সেখানে এক্সটেনশন নিজে বসে, তাই না থাকলে বুঝতে হবে কেউ সরিয়েছে বা বন্ধ করেছে। আর কোনো লিস্টে একটা পাতার ব্যতিক্রম (যেমন একটা YouTube চ্যানেল) থাকলে, এক্সটেনশন ছাড়া সব ব্রাউজার বন্ধ হয়, কারণ সেই সাইটের বাকিটা শুধু এক্সটেনশনই ব্লক রাখতে পারে।"],
      how: ["আগে \"এক্সটেনশন স্থায়ীভাবে বসানো\" চালু করে \"এখন চলা ব্রাউজার\" দেখুন: যে ব্রাউজার ব্যবহার করেন সবগুলোতে \"সুরক্ষিত\" দেখাতে হবে।",
        "তারপর এটা চালু করুন। যে ব্রাউজারে তখনো \"ফিল্টার হচ্ছে না – বন্ধ হবে\" দেখায়, ব্লকের সময় সেটা বন্ধ হবে।"],
      note: "সুরক্ষা চালু থাকলে বন্ধ করা যায় না।",
    },
    running_browsers: {
      t: "এখন চলা ব্রাউজার",
      p: ["এই কম্পিউটারে খোলা প্রতিটা ব্রাউজারের সরাসরি অবস্থা।",
        "সুরক্ষিত: এক্সটেনশন কাজ করছে। ফিল্টার হচ্ছে না – বন্ধ হবে: ব্লকের সময় বন্ধ হবে। ফিল্টার হচ্ছে না: এক্সটেনশন নেই, তবে ব্রাউজার সুরক্ষা বন্ধ।"],
    },
    safe_search: {
      t: "SafeSearch বাধ্যতামূলক",
      p: ["Google, Bing, DuckDuckGo আর YouTube-কে সবসময় সবচেয়ে কড়া নিরাপদ মোডে রাখে, ফলে অশ্লীল ছবি, ভিডিও আর ফলাফল সার্চে আসে না। এক্সটেনশন ছাড়াও এই কম্পিউটারের সব ব্রাউজার আর অ্যাপে কাজ করে, আর সার্চ পাতা থেকে বন্ধ করা যায় না।"],
    },
    private_windows: {
      t: "প্রাইভেট (ইনকগনিটো) উইন্ডো বন্ধ",
      p: ["Chrome, Edge, Brave, Vivaldi আর Firefox-এ Incognito / InPrivate উইন্ডো বন্ধ করে দেয়। প্রাইভেট উইন্ডোতে প্রায়ই এক্সটেনশন চলে না, তাই এটা বন্ধ করলে সেই ফাঁকটাও বন্ধ হয়।",
        "নতুন ইনস্টলে শুরু থেকেই চালু। সুরক্ষা চালু থাকলে ব্রাউজারের নিজস্ব \"secure DNS\"-ও বন্ধ থাকে, যাতে প্রতিটা সাইটের নাম ব্লক লিস্ট পেরিয়ে যায় (সাধারণ ব্রাউজিংয়ে কোনো প্রভাব নেই)।"],
      note: "সুরক্ষা চালু থাকলে বন্ধ করা যায় না।",
    },
    managed_ext: {
      t: "এক্সটেনশন স্থায়ীভাবে বসানো",
      p: ["স্কুল আর অফিস যে অফিসিয়াল ব্রাউজার পলিসি ব্যবহার করে, সেটাই ব্যবহার করে। এক্সটেনশন Chrome, Edge, Brave আর Firefox-এ নিজে বসে, সরানো বা বন্ধ করা যায় না, আর ব্রাউজার নতুন করে ইনস্টল করলেও ফিরে আসে।"],
      note: "ব্রাউজার সুরক্ষার সাথে নিজে থেকে চালু হয়। সুরক্ষা চালু থাকলে বন্ধ করা যায় না।",
    },
    browser_lists: {
      t: "সমর্থিত ব্রাউজার",
      p: ["সমর্থিত ব্রাউজারে এক্সটেনশন চলে, তাই সব ফিচার কাজ করে: সাইটের অংশ, শব্দ আর SafeSearch।",
        "অসমর্থিত ব্রাউজার ভেতর থেকে ফিল্টার করা যায় না। পুরো সাইট সেখানেও ব্লক হয়, আর ব্রাউজার সুরক্ষা চালু থাকলে ব্লকের সময় সেগুলো বন্ধ হয়।"],
    },
    filters: {
      t: "আংশিক ব্লক",
      p: ["সাইটের শুধু সবচেয়ে আসক্তিকর অংশটা লুকায়, বাকিটা চলে। যেমন সার্চ করে পাওয়া YouTube ভিডিও দেখা যাবে, কিন্তু Shorts আর অফুরন্ত সাজেশন থাকবে না।",
        "যে অংশগুলো লুকাতে চান টিক দিয়ে সেভ চাপুন। এক্সটেনশন থাকা সব ব্রাউজারে সবসময় চলে।"],
    },
    backup: {
      t: "ব্যাকআপ ও ফেরত আনা",
      p: ["আপনার সব লিস্ট, সময়সূচি, সীমা আর সেটিং লেখা আকারে। কপি করে নিরাপদ জায়গায় রাখুন, যেমন নিজেকে একটা ইমেইলে।",
        "নতুন কম্পিউটারে ইমপোর্ট চেপে পেস্ট করলে সব ফিরে আসে। সুরক্ষার নিয়ম তখনও চলে: ইমপোর্ট দিয়ে লক করা কিছু ঢিলা করা যায় না।"],
    },
    updates: {
      t: "আপডেট",
      p: ["নতুন ভার্সন এলে অ্যাপ নিজে থেকে আপডেট হয়। ইনস্টলের আগে প্রতিটা আপডেট আমাদের ডিজিটাল স্বাক্ষর দিয়ে যাচাই হয়, তাই নকল আপডেট বসানো যায় না।",
        "আপডেটে আপনার সব লিস্ট, সময়সূচি, সুরক্ষা আর Premium থেকে যায়। \"এখনই আপডেট\" অপেক্ষমাণ আপডেট সাথে সাথে বসায়।"],
    },
    extension: {
      t: "ব্রাউজার এক্সটেনশন",
      p: ["HTL Content Guard PC এক্সটেনশন ব্রাউজারের ভেতরে ফিল্টার করে: আংশিক ব্লক (Shorts, Reels, ফিড), শব্দ ব্লক, ব্লক পাতা আর SafeSearch।",
        "ব্রাউজার সুরক্ষা চালু থাকলে Chrome, Edge আর Brave-এ এটা নিজে বসে যায়। চাইলে Chrome Web Store-এ \"HTL Content Guard PC\" খুঁজে নিজেও যোগ করতে পারেন।"],
    },
  },
};

const UI = {
  en: { label: "How this works", how: "How to use it", note: "Good to know", english: "Show in English", translate: "Translate", close: "Close" },
  bn: { label: "কীভাবে কাজ করে", how: "কীভাবে ব্যবহার করবেন", note: "জেনে রাখুন", english: "English-এ দেখুন", translate: "অনুবাদ", close: "বন্ধ" },
};

/** The small (i) button placed next to a feature. */
export function tip(key) {
  if (!HELP.en[key]) return "";
  return `<button type="button" class="tip" data-tip="${key}" aria-label="${UI.en.label}" aria-expanded="false">i</button>`;
}

function text(entry) {
  return [entry.t, ...(entry.p || []), ...(entry.how || []), entry.note || ""].filter(Boolean).join("\n\n");
}

const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);

function body(key, lang) {
  const e = HELP[lang]?.[key] ?? HELP.en[key];
  const ui = UI[lang] ?? UI.en;
  return `<h3>${esc(e.t)}</h3>
    ${(e.p || []).map((x) => `<p>${esc(x)}</p>`).join("")}
    ${e.how ? `<strong>${esc(ui.how)}</strong><ol>${e.how.map((x) => `<li>${esc(x)}</li>`).join("")}</ol>` : ""}
    ${e.note ? `<p class="tip-note"><strong>${esc(ui.note)}:</strong> ${esc(e.note)}</p>` : ""}`;
}

let pop = null;
let current = null;
let shownLang = "en";

function close() {
  if (!pop) return;
  pop.remove();
  pop = null;
  current?.setAttribute("aria-expanded", "false");
  current = null;
}

function render() {
  const key = current.dataset.tip;
  const ui = getLanguage();
  const hasOwn = ui !== "en" && HELP[ui]?.[key];
  const rtl = langInfo(shownLang).dir === "rtl";
  // Machine translation for UI languages without a hand translation.
  const gt = ui !== "en" && !hasOwn
    ? `https://translate.google.com/?sl=en&tl=${encodeURIComponent(ui)}&op=translate&text=${encodeURIComponent(text(HELP.en[key]))}`
    : "";
  const toggle = hasOwn
    ? `<button type="button" class="btn small" data-tip-lang="${shownLang === "en" ? ui : "en"}">${esc(shownLang === "en" ? (UI[ui]?.translate ?? UI.en.translate) + " · " + langInfo(ui).name : UI[ui]?.english ?? UI.en.english)}</button>`
    : gt ? `<a class="btn small" href="${gt}" target="_blank" rel="noopener">${esc(UI.en.translate)} · ${esc(langInfo(ui).name)}</a>` : "";
  pop.innerHTML = `<div class="tip-body" dir="${rtl ? "rtl" : "ltr"}" lang="${shownLang}">${body(key, shownLang)}</div>
    <div class="row between tip-foot">${toggle}<button type="button" class="btn small" data-tip-close>${esc((UI[shownLang] ?? UI.en).close)}</button></div>`;
  place();
}

function place() {
  const r = current.getBoundingClientRect();
  const w = Math.min(400, window.innerWidth - 24);
  pop.style.width = `${w}px`;
  const left = Math.min(Math.max(12, r.left + r.width / 2 - w / 2), window.innerWidth - w - 12);
  pop.style.left = `${left}px`;
  const below = window.innerHeight - r.bottom;
  const h = pop.offsetHeight;
  const top = below >= h + 12 || below > r.top ? r.bottom + 8 : r.top - h - 8;
  // Always keep the whole box on screen (it scrolls inside when very long).
  pop.style.top = `${Math.max(12, Math.min(top, window.innerHeight - h - 12))}px`;
}

function open(btn) {
  close();
  current = btn;
  shownLang = "en";
  pop = document.createElement("div");
  pop.className = "tip-pop";
  pop.setAttribute("role", "dialog");
  pop.setAttribute("aria-label", HELP.en[btn.dataset.tip].t);
  document.body.appendChild(pop);
  btn.setAttribute("aria-expanded", "true");
  render();
}

/** Wire up every (i) button once, for the whole app (event delegation). */
export function initTips() {
  document.addEventListener("click", (e) => {
    const btn = e.target.closest?.("[data-tip]");
    if (btn) {
      e.preventDefault();
      e.stopPropagation();
      return btn === current ? close() : open(btn);
    }
    const lang = e.target.closest?.("[data-tip-lang]");
    if (lang) {
      shownLang = lang.dataset.tipLang;
      return render();
    }
    if (e.target.closest?.("[data-tip-close]")) return close();
    if (pop && !pop.contains(e.target)) close();
  }, true);
  // Capture phase, so Escape closes only the help and not a dialog behind it.
  window.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && pop) {
      e.stopPropagation();
      const b = current;
      close();
      b?.focus();
    }
  }, true);
  window.addEventListener("resize", () => pop && place());
  document.addEventListener("scroll", () => pop && place(), true);
}
