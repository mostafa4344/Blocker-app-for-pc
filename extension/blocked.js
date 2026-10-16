// Block page: explains what is blocked, until when, and reminds the user of their goal.
const ext = globalThis.browser ?? chrome;

const TEXT = {
  bn: {
    title: "এই সাইটটি এখন ব্লক করা",
    back: "পেছনে যান",
    open: "ContentGuard খুলুন",
    until: "পর্যন্ত",
    session: "ফোকাস সেশন চলছে",
    schedule: "সময়সূচি অনুযায়ী ব্লক",
    limit: "আজকের সময়সীমা শেষ",
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
    open: "Open ContentGuard",
    until: "until",
    session: "Focus session in progress",
    schedule: "Blocked by schedule",
    limit: "Today's time limit is used up",
    goal: "Your goal",
    quotes: [
      "Small focused moments build big results.",
      "What you control is the next minute.",
      "The feed can wait. Your goals can't.",
      "Deep work is the superpower of our time.",
      "Discipline is choosing what you want most over what you want now.",
    ],
  },
};

function fmt(iso, lang) {
  return new Date(iso).toLocaleString(lang === "bn" ? "bn-BD" : "en-US", { hour: "numeric", minute: "2-digit", weekday: "short" });
}

(async () => {
  const raw = location.hash.slice(1);
  let blockedUrl = raw;
  try {
    blockedUrl = decodeURIComponent(raw);
  } catch { /* already plain */ }
  const { policy } = await ext.storage.local.get("policy");
  const lang = policy?.language === "bn" ? "bn" : "en";
  const T = TEXT[lang];
  document.documentElement.lang = lang;
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
