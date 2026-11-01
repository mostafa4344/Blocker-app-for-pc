// Capture helper for the tutorial video: drives the demo app in a clean Chrome.
import puppeteer from "puppeteer-core";
import fs from "node:fs";

export const BASE = "http://127.0.0.1:17346/";
export const OUT = process.env.CAP_OUT || "C:/Users/user/AppData/Local/Temp/cgvideo/shots";
fs.mkdirSync(OUT, { recursive: true });

export const W = 1440, H = 810, DSF = 3840 / 1440;

export async function open() {
  const browser = await puppeteer.launch({
    executablePath: "C:/Program Files/Google/Chrome/Application/chrome.exe",
    headless: true,
    userDataDir: "C:/Users/user/AppData/Local/Temp/cgvideo/profile",
    args: ["--no-first-run", "--disable-extensions", "--hide-scrollbars", "--font-render-hinting=none", "--lang=en-US"],
    defaultViewport: { width: W, height: H, deviceScaleFactor: DSF },
  });
  const page = await browser.newPage();
  await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "light" }]);
  return { browser, page };
}

/** Saves a 4K PNG plus the CSS boxes of the named selectors (for highlights). */
export async function shot(page, name, marks = {}) {
  await new Promise((r) => setTimeout(r, 450));
  await page.evaluate(() => document.activeElement?.blur?.());
  const boxes = {};
  for (const [k, sel] of Object.entries(marks)) {
    const el = typeof sel === "string" ? await page.$(sel) : await sel;
    if (!el) { console.warn("no element", name, k, sel); continue; }
    const b = await el.boundingBox();
    if (b) boxes[k] = [b.x / W, b.y / H, b.width / W, b.height / H];
  }
  await page.screenshot({ path: `${OUT}/${name}.png` });
  fs.writeFileSync(`${OUT}/${name}.json`, JSON.stringify(boxes));
  console.log("shot", name, Object.keys(boxes).join(","));
}

/** Finds a clickable element by its visible text. */
export async function byText(page, text, sel = "button, a, label, summary, [role=button], li") {
  const handles = await page.$$(sel);
  for (const h of handles) {
    const t = await h.evaluate((e) => e.innerText.trim());
    const vis = await h.evaluate((e) => !!(e.offsetWidth || e.offsetHeight));
    if (vis && t.includes(text)) return h;
  }
  return null;
}

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
