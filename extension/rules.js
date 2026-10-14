// Builds declarativeNetRequest rules from the agent policy.
// Shared by the background worker and the unit tests (plain script, no imports).

const RULE_LIMIT = 900;

function escapeRe(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** Parses a site rule like the agent does: `facebook.com`, `*.reddit.com`, `youtube.com/shorts`, `*`. */
function parseSite(rule) {
  let r = String(rule).trim().toLowerCase();
  if (!r) return null;
  if (r === "*") return { domain: "", subOnly: false, path: null };
  r = r.replace(/^[a-z]+:\/\//, "");
  const slash = r.indexOf("/");
  let host = slash >= 0 ? r.slice(0, slash) : r;
  let path = slash >= 0 ? r.slice(slash).replace(/\*+$/, "") : null;
  if (path === "/" || path === "") path = null;
  let subOnly = false;
  if (host.startsWith("*.")) {
    subOnly = true;
    host = host.slice(2);
  }
  host = host.replace(/^www\./, "").replace(/\.$/, "");
  if (!host || /\s/.test(host)) return null;
  return { domain: host, subOnly, path };
}

function siteRegex(rule) {
  const p = parseSite(rule);
  if (!p) return null;
  if (!p.domain) return "^https?://.*";
  const sub = p.subOnly ? "([^/?#]+\\.)" : "([^/?#]*\\.)?";
  const tail = p.path ? `${escapeRe(p.path)}.*` : "([/?#].*)?";
  return `^https?://${sub}${escapeRe(p.domain)}(:[0-9]+)?${tail}$`;
}

function keywordRegex(k) {
  const kw = String(k).trim().toLowerCase();
  return kw ? `^https?://.*${escapeRe(kw)}.*$` : null;
}

/**
 * @param policy  agent policy (`blocked_sites`, `keywords`, `allowlist_active`, `allowed_sites`)
 * @param blockedPage  absolute URL of blocked.html
 */
function buildRules(policy, blockedPage) {
  const rules = [];
  let id = 1;
  const redirect = (regexFilter, priority) => ({
    id: id++,
    priority,
    action: { type: "redirect", redirect: { regexSubstitution: `${blockedPage}#\\0` } },
    condition: { regexFilter, resourceTypes: ["main_frame"], isUrlFilterCaseSensitive: false },
  });
  if (!policy) return rules;
  if (policy.allowlist_active) {
    rules.push(redirect("^https?://.*$", 1));
    for (const s of policy.allowed_sites || []) {
      const re = siteRegex(s);
      if (re) rules.push({ id: id++, priority: 2, action: { type: "allow" }, condition: { regexFilter: re, resourceTypes: ["main_frame"], isUrlFilterCaseSensitive: false } });
    }
    rules.push({ id: id++, priority: 2, action: { type: "allow" }, condition: { regexFilter: "^https?://(localhost|127\\.0\\.0\\.1)(:[0-9]+)?([/?#].*)?$", resourceTypes: ["main_frame"] } });
  }
  for (const s of policy.blocked_sites || []) {
    const re = siteRegex(s);
    if (re) rules.push(redirect(re, 3));
  }
  for (const k of policy.keywords || []) {
    const re = keywordRegex(k);
    if (re) rules.push(redirect(re, 3));
  }
  return rules.slice(0, RULE_LIMIT);
}

if (typeof module !== "undefined") module.exports = { parseSite, siteRegex, keywordRegex, buildRules };
