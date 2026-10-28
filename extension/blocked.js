// Block page: explains what is blocked, until when, and reminds the user of their goal.
const ext = globalThis.browser ?? chrome;

const TEXT = {
  bn: {
    title: "এই সাইটটি এখন ব্লক করা",
    back: "পেছনে যান",
    open: "HTL Content Guard খুলুন",
    until: "পর্যন্ত",
    session: "ফোকাস সেশন চলছে",
    schedule: "সময়সূচি অনুযায়ী ব্লক",
    limit: "আজকের সময়সীমা শেষ",
    always: "সবসময় ব্লক করা",
    goal: "আপনার লক্ষ্য",
    quotes: [
      "ছোট ছোট মনোযোগী মুহূর্তই বড় সাফল্য তৈরি করে।",
      "আপনি যা নিয়ন্ত্রণ করতে পারেন, তা হলো আপনার পরবর্তী এক মিনিট।",
      "ফোন আর স্ক্রল অপেক্ষা করতে পারে, আপনার স্বপ্ন নয়।",
      "আজকের পরিশ্রম কালকের স্বস্তি।",
      "মনোযোগই নতুন সুপারপাওয়ার।",
    ],
  },
  en: {
    title: "This site is blocked right now",
    back: "Go back",
    open: "Open HTL Content Guard",
    until: "until",
    session: "Focus session in progress",
    schedule: "Blocked by schedule",
    limit: "Today's time limit is used up",
    always: "Always blocked",
    goal: "Your goal",
    quotes: [
      "Small focused moments build big results.",
      "What you control is the next minute.",
      "The feed can wait. Your goals can't.",
      "Deep work is the superpower of our time.",
      "Discipline is choosing what you want most over what you want now.",
    ],
  },
  ar: {
    title: "هذا الموقع محظور الآن",
    back: "رجوع",
    open: "افتح HTL Content Guard",
    until: "حتى",
    session: "جلسة تركيز جارية",
    schedule: "محظور حسب الجدول",
    limit: "انتهى وقت اليوم",
    always: "محظور دائمًا",
    goal: "هدفك",
    quotes: [
      "اللحظات الصغيرة المركّزة تصنع نتائج كبيرة.",
      "ما تملكه هو الدقيقة القادمة.",
      "قُل للمؤمنين يغضّوا من أبصارهم — النور: 30",
      "الانضباط أن تختار ما تريده أكثر على ما تريده الآن.",
    ],
  },
  ur: {
    title: "یہ سائٹ ابھی بلاک ہے",
    back: "واپس جائیں",
    open: "HTL Content Guard کھولیں",
    until: "تک",
    session: "فوکس سیشن جاری ہے",
    schedule: "شیڈول کے مطابق بلاک",
    limit: "آج کا وقت ختم ہو گیا",
    always: "ہمیشہ بلاک",
    goal: "آپ کا مقصد",
    quotes: [
      "چھوٹے چھوٹے توجہ والے لمحے بڑی کامیابی بناتے ہیں۔",
      "آپ کے اختیار میں صرف اگلا ایک منٹ ہے۔",
      "ایمان والوں سے کہہ دو کہ اپنی نگاہیں نیچی رکھیں — النور: 30",
      "فیڈ انتظار کر سکتی ہے، آپ کے خواب نہیں۔",
    ],
  },
  id: {
    title: "Situs ini sedang diblokir",
    back: "Kembali",
    open: "Buka HTL Content Guard",
    until: "sampai",
    session: "Sesi fokus sedang berjalan",
    schedule: "Diblokir sesuai jadwal",
    limit: "Batas waktu hari ini sudah habis",
    always: "Selalu diblokir",
    goal: "Tujuan Anda",
    quotes: [
      "Momen fokus yang kecil membangun hasil yang besar.",
      "Yang Anda kendalikan adalah satu menit berikutnya.",
      "Feed bisa menunggu. Impian Anda tidak.",
      "Disiplin adalah memilih yang paling Anda inginkan, bukan yang Anda inginkan sekarang.",
    ],
  },
  tr: {
    title: "Bu site şu anda engelli",
    back: "Geri dön",
    open: "HTL Content Guard'ı aç",
    until: "kadar",
    session: "Odak oturumu sürüyor",
    schedule: "Zamanlamaya göre engelli",
    limit: "Bugünkü süre doldu",
    always: "Her zaman engelli",
    goal: "Hedefiniz",
    quotes: [
      "Küçük odaklı anlar büyük sonuçlar doğurur.",
      "Kontrol edebildiğin şey bir sonraki dakika.",
      "Akış bekleyebilir. Hedeflerin bekleyemez.",
      "Disiplin, en çok istediğini şu an istediğine tercih etmektir.",
    ],
  },
};
const LOCALES = { bn: "bn-BD", en: "en-US", ar: "ar", ur: "ur-PK", id: "id-ID", tr: "tr-TR" };

function fmt(iso, lang) {
  return new Date(iso).toLocaleString(LOCALES[lang] ?? "en-US", { hour: "numeric", minute: "2-digit", weekday: "short" });
}

(async () => {
  const raw = location.hash.slice(1);
  let blockedUrl = raw;
  try {
    blockedUrl = decodeURIComponent(raw);
  } catch { /* already plain */ }
  const { policy } = await ext.storage.local.get("policy");
  const lang = TEXT[policy?.language] ? policy.language : "en";
  const T = TEXT[lang];
  document.documentElement.lang = lang;
  document.documentElement.dir = lang === "ar" || lang === "ur" ? "rtl" : "ltr";
  document.getElementById("title").textContent = T.title;
  document.getElementById("back").textContent = T.back;
  document.getElementById("open").textContent = T.open;
  let host = "";
  try {
    host = new URL(blockedUrl).hostname;
  } catch { /* not a URL */ }
  document.getElementById("site").textContent = host;
  const a = policy?.active?.[0];
  if (a) {
    const until = a.until ? ` · ${fmt(a.until, lang)} ${T.until}` : a.until_local ? ` · ${fmt(a.until_local, lang)} ${T.until}` : "";
    document.getElementById("reason").textContent = `${T[a.reason] ?? ""}${until}${a.locked ? " 🔒" : ""}`;
  }
  if (policy?.goal) {
    const g = document.getElementById("goal");
    g.hidden = false;
    g.textContent = `${T.goal}: ${policy.goal}`;
  }
  document.getElementById("quote").textContent = T.quotes[Math.floor(Math.random() * T.quotes.length)];
  document.getElementById("back").addEventListener("click", () => {
    if (history.length > 2) history.go(-2);
    else window.close();
  });
  // Count the attempt on the dashboard.
  if (/^https?:/.test(blockedUrl)) {
    fetch("http://127.0.0.1:17345/api/ext/activity", {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-ContentGuard": "1" },
      body: JSON.stringify({ url: blockedUrl, seconds: 0, opened: true, browser: navigator.userAgent.includes("Firefox") ? "firefox" : "chromium" }),
    }).catch(() => {});
  }
})();
