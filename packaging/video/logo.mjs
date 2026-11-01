import { open } from "./cap.mjs";
import fs from "node:fs";
const svg = fs.readFileSync("C:/Users/user/HTL-Content-Guard/Blocker-app-for-pc-ccr-00759c63-vxxesd/ui/icon.svg", "utf8");
const { browser, page } = await open();
await page.setViewport({ width: 512, height: 512, deviceScaleFactor: 2 });
await page.setContent(`<style>body{margin:0;background:transparent}svg{width:512px;height:512px;display:block}</style>${svg}`);
await (await page.$("svg")).screenshot({ path: "C:/Users/user/AppData/Local/Temp/cgvideo/logo.png", omitBackground: true });
await browser.close();
