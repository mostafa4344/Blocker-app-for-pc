// Walks the demo app like a new user and saves every frame the video needs.
import { open, BASE, sleep, shot, byText, W, H } from "./cap.mjs";

const { browser, page } = await open();
const go = async (p) => { await page.click(`#nav [data-page=${p}]`); await sleep(900); await page.evaluate(() => window.scrollTo(0, 0)); };
const type = async (sel, text) => { await page.click(sel); await page.keyboard.type(text, { delay: 8 }); };
const scrollTo = async (sel, offset = 90) => {
  await page.evaluate((s, o) => { const e = document.querySelector(s); const m = document.querySelector("#main"); const y = e.getBoundingClientRect().top + window.scrollY - o; window.scrollTo(0, y); m?.scrollTo?.(0, m.scrollTop + e.getBoundingClientRect().top - o); }, sel, offset);
  await sleep(400);
};
// Pretend the Chrome extension is installed and talking to the app (it reports every 30 s).
let extOn = false;
const ext = setInterval(() => { if (extOn) fetch("http://127.0.0.1:17346/api/ext/policy?browser=chromium&brand=chrome").catch(() => {}); }, 3000);

const only = process.argv[2];
const want = (k) => !only || only.split(",").includes(k);

// ---------- website ----------
if (want("web")) {
  const site = "file:///C:/Users/user/HTL-Content-Guard/Blocker-app-for-pc-ccr-00759c63-vxxesd/web/content-guard-pc-free-trial-page.html";
  await page.setContent(`<!doctype html><meta name=viewport content="width=device-width,initial-scale=1"><body style="margin:0">` +
    (await (await fetch("data:,")).text()), { waitUntil: "load" }).catch(() => {});
  await page.goto(site, { waitUntil: "load" });
  await shot(page, "web_hero", { dl: "#cg-download .cg-btn" });
  await page.evaluate(() => document.querySelector(".cg-trial").scrollIntoView({ block: "center" }));
  await shot(page, "web_trial", { box: ".cg-trial" });
  await page.evaluate(() => document.querySelector("#cg-features").scrollIntoView());
  await page.evaluate(() => window.scrollBy(0, -70));
  await shot(page, "web_features", { grid: ".cg-grid" });
}

// ---------- first start / onboarding ----------
await page.goto(BASE, { waitUntil: "networkidle0" });
await sleep(1500);
if (want("ob")) {
  await shot(page, "ob1", { lang: "#ob-lang", goal: "#ob-goal" });
  await type("#ob-goal", "Stay focused on my studies and protect my eyes");
  await shot(page, "ob1_typed", { goal: "#ob-goal", next: "#ob-next" });
  await page.click("#ob-next"); await sleep(800);
  await shot(page, "ob2", { grid: ".preset-grid", adult: "[data-p=adult]", gambling: "[data-p=gambling]", shorts: "[data-p=shorts]", social: "[data-p=social]" });
  await page.click("[data-p=social]"); await sleep(300);
  await shot(page, "ob2_social", { social: "[data-p=social]", next: "#ob-next" });
  await page.click("#ob-next"); await sleep(800);
  await shot(page, "ob3", { off: "[data-lv=off]", mindful: "[data-lv=mindful]", committed: "[data-lv=committed]", finish: "#ob-next" });
  await page.click("#ob-next"); await sleep(2500);
  await shot(page, "home_first", { banner: ".banner.info", main: "#main .card" });
}

// ---------- extension ----------
if (want("ext")) {
  await go("settings");
  await scrollTo("#s-managed", 260);
  await shot(page, "settings_browsers", { managed: "#s-managed", reqext: "#s-reqext", private: "#s-private" });
  extOn = true; await sleep(4500);
  await go("home");
  await shot(page, "home_ext_on", { main: "#main" });
}

// ---------- block lists ----------
if (want("lists")) {
  await go("lists");
  await shot(page, "lists", { preset: "#add-preset", add: "#add-list" });
  await page.click("#add-preset"); await sleep(700);
  await shot(page, "lists_presets", { grid: ".modal .preset-grid, dialog .preset-grid" });
  await page.keyboard.press("Escape"); await sleep(500);
  await page.click("#add-list"); await sleep(700);
  await type("#l-name", "Study time");
  await type("#l-sites", "facebook.com\ninstagram.com\ntiktok.com");
  await type("#l-apps", "steam.exe");
  await type("#l-keywords", "trailer\nmeme");
  await shot(page, "list_new", { name: "#l-name", sites: "#l-sites", apps: "#l-apps", words: "#l-keywords", always: "#l-always", save: "#l-save" });
  await page.click("#l-save"); await sleep(1200);
  await shot(page, "lists_after", { main: "#main" });
}

// ---------- focus session ----------
if (want("focus")) {
  await go("home");
  await scrollTo("#q-start", 520);
  const study = await byText(page, "Study time", "[data-pick]");
  if (study) { await study.click(); await sleep(300); }
  const shorts = await byText(page, "Short videos", "[data-pick]");
  if (shorts && (await shorts.evaluate((e) => e.getAttribute("aria-pressed"))) === "true") { await shorts.click(); await sleep(300); }
  const h1 = await byText(page, "1 h", "[data-min]");
  if (h1) { await h1.click(); await sleep(300); }
  await shot(page, "focus_setup", { pick: study ?? "[data-pick]", dur: h1 ?? "[data-min]", lock: "#q-lock", summary: "#q-summary", start: "#q-start" });
  await page.click("#q-start"); await sleep(1800);
  await page.evaluate(() => window.scrollTo(0, 0));
  await shot(page, "focus_running", { main: "#main .card" });
}

// ---------- schedules ----------
if (want("sched")) {
  await go("schedules");
  await page.click("#add"); await sleep(700);
  await page.select("#s-list", await page.$eval("#s-list", (s) => [...s.options].find((o) => o.text.includes("Social"))?.value || s.value));
  const tpl = await page.$$("[data-template]");
  if (tpl[0]) { await tpl[0].click(); await sleep(300); }
  await shot(page, "sched_new", { list: "#s-list", tpl: "[data-template]", days: "[data-day]", start: "#s-start", end: "#s-end", save: "#s-save" });
  await page.click("#s-save"); await sleep(1200);
  await shot(page, "sched_after", { main: "#main" });
}

// ---------- daily limits ----------
if (want("limits")) {
  await go("limits");
  await page.click("#add"); await sleep(700);
  await page.select("#m-list", await page.$eval("#m-list", (s) => [...s.options].find((o) => o.text.includes("Short"))?.value || s.value));
  await page.click("#m-min", { clickCount: 3 }); await page.keyboard.type("30");
  await shot(page, "limit_new", { list: "#m-list", min: "#m-min", opens: "#m-opens", save: "#m-save" });
  await page.click("#m-save"); await sleep(1200);
  await shot(page, "limits_after", { main: "#main" });
}

// ---------- settings: SafeSearch + partial blocking ----------
if (want("settings")) {
  await go("settings");
  await scrollTo("#s-safe", 200);
  await shot(page, "settings_safe", { safe: "#s-safe", private: "#s-private" });
  const yt = await byText(page, "Hide YouTube Shorts", "label");
  if (yt) await yt.evaluate((e) => e.scrollIntoView({ block: "center" }));
  await sleep(400);
  await shot(page, "settings_partial", { yt: yt ?? "label" });
}

// ---------- protection ----------
if (want("prot")) {
  await go("protection");
  await shot(page, "protection", { off: "[data-level=off]", mindful: "[data-level=mindful]", committed: "[data-level=committed]", cool: "#p-cool" });
  await scrollTo("#u-req", 300);
  await shot(page, "protection_uninstall", { req: "#u-req" });
}

// ---------- premium ----------
if (want("premium")) {
  await go("home");
  await page.evaluate(() => window.scrollTo(0, 0));
  await shot(page, "trial_banner", { banner: ".banner.info" });
  await go("premium");
  await shot(page, "premium", { main: "#main" });
  const yearly = await byText(page, "Yearly", "[data-plan]");
  if (yearly) { await yearly.click(); await sleep(400); }
  await shot(page, "premium_plan", { plan: yearly ?? "[data-plan]" });
  await scrollTo("#pay-num", 260);
  await shot(page, "premium_pay", { method: "[data-method]", num: "#pay-num", copy: "#copy-num" });
  await type("#trx", "9KD7XQ2M4P");
  await type("#contact", "01700000000");
  await scrollTo("#trx", 260);
  await shot(page, "premium_trx", { trx: "#trx", contact: "#contact", send: "#send" });
  await page.click("#send"); await sleep(1800);
  await shot(page, "premium_sent", { main: "#main" });
  await scrollTo("#dev-id", 380);
  await shot(page, "premium_devid", { dev: "#dev-id", copy: "#copy-id" });
}

clearInterval(ext);
await browser.close();
console.log("capture done");
