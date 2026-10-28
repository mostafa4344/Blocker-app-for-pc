const ext = globalThis.browser ?? chrome;

const TEXT = {
  bn: { offline: "HTL Content Guard সার্ভিস পাওয়া যাচ্ছে না", blocking: "ব্লক চলছে: ", nothing: "এখন কিছু ব্লক হচ্ছে না", open: "HTL Content Guard খুলুন" },
  en: { offline: "HTL Content Guard service not reachable", blocking: "Blocking: ", nothing: "Nothing blocked right now", open: "Open HTL Content Guard" },
  ar: { offline: "تعذّر الوصول إلى خدمة HTL Content Guard", blocking: "الحظر يعمل: ", nothing: "لا شيء محظور الآن", open: "افتح HTL Content Guard" },
  ur: { offline: "HTL Content Guard سروس تک رسائی نہیں", blocking: "بلاکنگ جاری: ", nothing: "ابھی کچھ بلاک نہیں", open: "HTL Content Guard کھولیں" },
  id: { offline: "Layanan HTL Content Guard tidak terjangkau", blocking: "Memblokir: ", nothing: "Tidak ada yang diblokir saat ini", open: "Buka HTL Content Guard" },
  tr: { offline: "HTL Content Guard hizmetine ulaşılamıyor", blocking: "Engelleniyor: ", nothing: "Şu anda hiçbir şey engellenmiyor", open: "HTL Content Guard'ı aç" },
};

ext.storage.local.get(["policy", "agentOnline"]).then(({ policy, agentOnline }) => {
  const lang = TEXT[policy?.language] ? policy.language : "en";
  const T = TEXT[lang];
  document.documentElement.lang = lang;
  document.documentElement.dir = lang === "ar" || lang === "ur" ? "rtl" : "ltr";
  const s = document.getElementById("s");
  if (!agentOnline) s.textContent = T.offline;
  else if (policy?.active?.length) s.textContent = T.blocking + policy.active.map((a) => a.block_list_name).join(", ");
  else s.textContent = T.nothing;
  document.getElementById("o").textContent = T.open;
});
