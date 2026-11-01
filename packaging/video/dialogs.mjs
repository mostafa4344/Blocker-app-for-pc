// Faithful illustrations of the two Windows prompts (they can't be screenshotted:
// SmartScreen/UAC run on a protected desktop).
import { open, sleep } from "./cap.mjs";
import fs from "node:fs";

const css = `
*{box-sizing:border-box;margin:0}
body{background:transparent;font-family:"Segoe UI",system-ui,sans-serif;-webkit-font-smoothing:antialiased}
.ss{width:640px;height:520px;background:#0b5cad;color:#fff;padding:40px 44px;position:relative;box-shadow:0 0 0 1px #2a78c8}
.ss .x{position:absolute;top:10px;right:16px;font-size:20px;opacity:.9}
.ss h1{font-weight:300;font-size:34px;letter-spacing:-.2px;margin:6px 0 22px}
.ss p{font-size:15px;line-height:1.5;margin-bottom:14px;max-width:540px}
.ss a{color:#fff;text-decoration:underline;font-size:15px}
.ss .info{font-size:15px;line-height:1.9;margin-top:18px}
.ss .info b{display:inline-block;width:90px;font-weight:400}
.ss .btns{position:absolute;right:28px;bottom:26px;display:flex;gap:10px}
.ss button{background:transparent;color:#fff;border:2px solid #fff;font:15px "Segoe UI";padding:7px 22px;min-width:120px}
.uac{width:460px;background:#f2f2f2;box-shadow:0 0 0 1px #8a8a8a,0 12px 40px rgba(0,0,0,.45)}
.uac .head{background:#f2b705;color:#000;padding:10px 16px 12px;font-size:15px}
.uac .body{background:#fff;padding:18px 20px 16px}
.uac h2{font-size:19px;font-weight:600;line-height:1.35;color:#000;margin-bottom:18px}
.uac .app{display:flex;gap:12px;align-items:center;margin-bottom:16px}
.uac .ic{width:34px;height:34px;border-radius:7px;background:#4f46e5;display:grid;place-items:center;color:#fff;font-size:20px}
.uac .app b{font-size:15px;font-weight:600}
.uac .meta{font-size:14px;color:#222;line-height:1.7}
.uac a{display:block;color:#0063b1;font-size:14px;margin:14px 0 4px}
.uac .foot{display:flex;gap:10px;padding:14px 20px 16px;background:#f2f2f2}
.uac button{flex:1;font:15px "Segoe UI";padding:7px;background:#ccc;border:2px solid #ccc;color:#000}
.uac button.focus{border-color:#0078d7}
`;
const smart = (expanded) => `<div class="ss" id="d"><span class="x">✕</span>
  <h1>Windows protected your PC</h1>
  <p>Microsoft Defender SmartScreen prevented an unrecognized app from starting. Running this app might put your PC at risk.</p>
  ${expanded
    ? `<div class="info"><div><b>App:</b> HTL-Content-Guard-PC-Setup.exe</div><div><b>Publisher:</b> Unknown publisher</div></div>
       <div class="btns"><button id="run">Run anyway</button><button>Don't run</button></div>`
    : `<a id="more">More info</a><div class="btns"><button>Don't run</button></div>`}
</div>`;
const uac = `<div class="uac" id="d"><div class="head">User Account Control</div>
  <div class="body"><h2>Do you want to allow this app from an unknown publisher to make changes to your device?</h2>
  <div class="app"><div class="ic">🛡️</div><b>HTL-Content-Guard-PC-Setup.exe</b></div>
  <div class="meta">Publisher: Unknown<br>File origin: Hard drive on this computer</div>
  <a>Show more details</a></div>
  <div class="foot"><button id="yes">Yes</button><button class="focus">No</button></div></div>`;

const { browser, page } = await open();
const OUT = "C:/Users/user/AppData/Local/Temp/cgvideo/shots";
for (const [name, html, mark] of [["ss1", smart(false), "#more"], ["ss2", smart(true), "#run"], ["uac", uac, "#yes"]]) {
  await page.setContent(`<style>${css}</style><div style="padding:30px;display:inline-block">${html}</div>`);
  await sleep(300);
  const d = await page.$("#d");
  const db = await d.boundingBox();
  const m = await (await page.$(mark)).boundingBox();
  await d.screenshot({ path: `${OUT}/${name}.png`, omitBackground: true });
  fs.writeFileSync(`${OUT}/${name}.json`, JSON.stringify({ btn: [(m.x - db.x) / db.width, (m.y - db.y) / db.height, m.width / db.width, m.height / db.height] }));
  console.log(name);
}
await browser.close();
