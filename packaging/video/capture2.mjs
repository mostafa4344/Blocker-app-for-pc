import { open, BASE, sleep, shot, byText, W, H } from "./cap.mjs";
import fs from "node:fs";

const { browser, page } = await open();
const only = process.argv[2];
const want = (k) => !only || only.split(",").includes(k);
const go = async (p) => { await page.click(`#nav [data-page=${p}]`); await sleep(900); await page.evaluate(() => window.scrollTo(0, 0)); };
const ext = setInterval(() => fetch("http://127.0.0.1:17346/api/ext/policy?browser=chromium&brand=chrome").catch(() => {}), 3000);

if (want("after")) {
  await page.goto(BASE, { waitUntil: "networkidle0" }); await sleep(1500);
  await go("premium");
  await shot(page, "premium_active", { status: "#main .card" });
  await go("home");
  await shot(page, "home_premium", { main: "#main .card" });
  await go("protection");
  const mindful = await page.evaluateHandle(() => [...document.querySelectorAll("#main .card, #main [class*=level] > *")].find((e) => e.innerText.startsWith("Mindful") && e.offsetHeight < 300));
  await shot(page, "protection2", { mindful, cool: "#p-cool" });
  await page.click("#u-req"); await sleep(900);
  await shot(page, "uninstall_modal", { modal: ".modal, dialog" });
}

if (want("block")) {
  // The extension's real block page, fed the demo app's live rules.
  const r = await fetch("http://127.0.0.1:17346/api/ext/policy?browser=chromium&brand=chrome");
  const policy = await r.json();
  const dir = "C:/Users/user/HTL-Content-Guard/Blocker-app-for-pc-ccr-00759c63-vxxesd/extension/";
  await page.setRequestInterception(true);
  page.on("request", (q) => (q.url().includes("127.0.0.1:17345") ? q.abort() : q.continue()));
  await page.evaluateOnNewDocument((p) => {
    globalThis.chrome = { storage: { local: { get: async () => ({ policy: p }) } } };
    Math.random = () => 0.05; // first quote
  }, policy);
  await page.goto("file:///" + dir + "blocked.html#https://www.facebook.com/", { waitUntil: "load" });
  await sleep(800);
  await shot(page, "blockpage", { title: "#title", reason: "#reason", goal: "#goal" });
  await page.setRequestInterception(false);
}

if (want("store")) {
  await page.goto("https://chromewebstore.google.com/detail/amlkhggimheblaboilcbjdjomlceckdp?hl=en", { waitUntil: "networkidle2", timeout: 45000 });
  await sleep(2500);
  // Hide notices that only exist on this PC (its policy) or change daily (user count).
  await page.evaluate(() => {
    for (const e of document.querySelectorAll("*")) {
      const t = e.innerText?.trim() || "";
      if (t === "This item's installation is required by your organization" && e.offsetHeight < 120) e.closest("section, div").style.visibility = "hidden";
      if (/^\d[\d,]* users?$/.test(t) && e.children.length === 0) e.style.visibility = "hidden";
    }
  });
  const add = await byText(page, "Add to Chrome", "button, a, span");
  await shot(page, "store", add ? { add } : {});
  console.log((await page.evaluate(() => document.body.innerText)).slice(0, 400));
}

clearInterval(ext);
await browser.close();
