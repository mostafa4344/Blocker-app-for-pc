const ext = globalThis.browser ?? chrome;
ext.storage.local.get(["policy", "agentOnline"]).then(({ policy, agentOnline }) => {
  const bn = policy?.language !== "en";
  const s = document.getElementById("s");
  if (!agentOnline) s.textContent = bn ? "ContentGuard সার্ভিস পাওয়া যাচ্ছে না" : "ContentGuard service not reachable";
  else if (policy?.active?.length) s.textContent = (bn ? "ব্লক চলছে: " : "Blocking: ") + policy.active.map((a) => a.block_list_name).join(", ");
  else s.textContent = bn ? "এখন কিছু ব্লক হচ্ছে না" : "Nothing blocked right now";
  document.getElementById("o").textContent = bn ? "ContentGuard খুলুন" : "Open ContentGuard";
});
