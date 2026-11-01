import { open, BASE, sleep, shot, byText } from "./cap.mjs";
const { browser, page } = await open();
const go = async (p) => { await page.click(`#nav [data-page=${p}]`); await sleep(900); await page.evaluate(() => window.scrollTo(0, 0)); };
const scrollTo = async (sel, offset) => {
  await page.evaluate((s, o) => { const e = document.querySelector(s); window.scrollTo(0, e.getBoundingClientRect().top + window.scrollY - o); }, sel, offset);
  await sleep(400);
};
const row = (id) => page.evaluateHandle((i) => document.getElementById(i).closest("label"), id);
const only = process.argv[2];
const want = (k) => !only || only.split(",").includes(k);

if (want("settings")) {
  await page.goto(BASE, { waitUntil: "networkidle0" }); await sleep(1500);
  await go("settings");
  await scrollTo("#s-managed", 260);
  await shot(page, "settings_browsers", { managed: await row("s-managed") });
  await scrollTo("#s-safe", 200);
  await shot(page, "settings_safe", { safe: await row("s-safe"), private: await row("s-private") });
  const yt = await byText(page, "Hide YouTube Shorts", "label");
  await yt.evaluate((e) => e.scrollIntoView({ block: "center" }));
  await sleep(400);
  // ring around the whole "Partial blocking" group
  const grp = await yt.evaluateHandle((e) => e.closest("section, .card, fieldset") || e.parentElement);
  await shot(page, "settings_partial", { yt: grp });
}

if (want("store")) {
  await page.goto("https://chromewebstore.google.com/detail/amlkhggimheblaboilcbjdjomlceckdp?hl=en", { waitUntil: "networkidle2", timeout: 45000 });
  await sleep(2500);
  await page.evaluate(() => {
    const all = [...document.querySelectorAll("body *")];
    const leaf = (re) => all.filter((e) => re.test(e.innerText?.trim() || "") && ![...e.children].some((c) => re.test(c.innerText?.trim() || "")));
    // "2 users" is loose text inside the chips row: blank those text nodes.
    for (const d of all.filter((e) => /\d\s*users?\s*$/i.test(e.textContent) && e.textContent.length < 60)) {
      for (const n of d.childNodes) if (n.nodeType === 3 && /users?|^\s*\d[\d,.]*\s*$/i.test(n.textContent)) n.textContent = "";
    }
    for (const e of leaf(/installation is required by your organization/i)) {
      let p = e;
      while (p.parentElement && p.offsetHeight < 40) p = p.parentElement;
      while (p.parentElement && p.parentElement.offsetHeight < 90) p = p.parentElement;
      p.style.display = "none";
    }
  });
  const add = await byText(page, "Add to Chrome", "button, a, span");
  await shot(page, "store", { add });
}
await browser.close();
