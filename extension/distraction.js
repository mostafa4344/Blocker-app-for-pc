// Applies the user's partial-blocking filters (Shorts, Reels, feeds) on supported sites.
(() => {
  const ext = globalThis.browser ?? chrome;
  let filters = [];

  function pathClass() {
    const p = location.pathname;
    if (p === "/" || p === "") return "ff-path-root";
    if (p === "/home") return "ff-path-home";
    if (p.startsWith("/feed")) return "ff-path-feed";
    return "";
  }

  function redirects() {
    const host = location.hostname;
    const p = location.pathname;
    if (filters.includes("youtube_shorts") && host.endsWith("youtube.com") && p.startsWith("/shorts/")) {
      const id = p.split("/")[2];
      if (id) location.replace(`/watch?v=${encodeURIComponent(id)}`);
    }
    if (filters.includes("instagram_reels") && host.endsWith("instagram.com") && /^\/(reels|explore)(\/|$)/.test(p)) {
      location.replace("/");
    }
    if (filters.includes("facebook_reels") && host.endsWith("facebook.com") && /^\/(reel|reels)(\/|$)/.test(p)) {
      location.replace("/");
    }
  }

  function apply() {
    const root = document.documentElement;
    if (!root) return;
    for (const c of [...root.classList]) if (c.startsWith("ff-")) root.classList.remove(c);
    for (const f of filters) root.classList.add(`ff-${f}`);
    const pc = pathClass();
    if (pc) root.classList.add(pc);
    redirects();
  }

  ext.storage.local.get("policy").then(({ policy }) => {
    filters = policy?.distraction_filters ?? [];
    apply();
  });
  ext.storage.onChanged.addListener((changes) => {
    if (changes.policy) {
      filters = changes.policy.newValue?.distraction_filters ?? [];
      apply();
    }
  });
  // Single-page apps change the URL without reloading.
  let last = location.href;
  setInterval(() => {
    if (location.href !== last) {
      last = location.href;
      apply();
    }
  }, 700);
})();
