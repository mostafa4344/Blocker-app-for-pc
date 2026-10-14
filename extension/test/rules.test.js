const test = require("node:test");
const assert = require("node:assert");
const { siteRegex, keywordRegex, buildRules } = require("../rules.js");

const matches = (re, url) => new RegExp(re, "i").test(url);

test("domain rules match the domain and its subdomains", () => {
  const re = siteRegex("facebook.com");
  assert.ok(matches(re, "https://facebook.com/"));
  assert.ok(matches(re, "https://www.facebook.com/feed?x=1"));
  assert.ok(matches(re, "http://m.facebook.com"));
  assert.ok(!matches(re, "https://notfacebook.com/"));
  assert.ok(!matches(re, "https://facebook.com.evil.io/"));
});

test("wildcard and path rules", () => {
  assert.ok(matches(siteRegex("*.reddit.com"), "https://old.reddit.com/r/x"));
  assert.ok(!matches(siteRegex("*.reddit.com"), "https://reddit.com/"));
  const shorts = siteRegex("youtube.com/shorts");
  assert.ok(matches(shorts, "https://www.youtube.com/shorts/abc"));
  assert.ok(!matches(shorts, "https://www.youtube.com/watch?v=1"));
  assert.ok(matches(siteRegex("https://www.x.com/"), "https://x.com/home"));
});

test("keywords", () => {
  assert.ok(matches(keywordRegex("casino"), "https://a.com/best-CASINO-games"));
  assert.equal(keywordRegex("  "), null);
});

test("allowlist rules", () => {
  const rules = buildRules({ allowlist_active: true, allowed_sites: ["github.com"], blocked_sites: [], keywords: [] }, "chrome-extension://id/blocked.html");
  assert.equal(rules[0].action.type, "redirect");
  assert.ok(rules.some((r) => r.action.type === "allow" && matches(r.condition.regexFilter, "https://github.com/x")));
  assert.ok(rules.some((r) => r.action.type === "allow" && matches(r.condition.regexFilter, "http://127.0.0.1:17345/")));
  const ids = rules.map((r) => r.id);
  assert.equal(new Set(ids).size, ids.length);
});
