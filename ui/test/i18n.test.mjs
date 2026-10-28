// Every UI language must have exactly the English keys, keep the same
// {placeholders}, and the bn/ar/... files must be valid modules.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const ui = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
// i18n.js imports "/lang-xx.js" (absolute, as served by the agent): rewrite for Node.
async function load() {
  const src = readFileSync(path.join(ui, "i18n.js"), "utf8").replace(
    /from "\/(lang-\w+\.js)"/g,
    (_, f) => `from "${pathToFileURL(path.join(ui, f)).href}"`,
  );
  return import(`data:text/javascript,${encodeURIComponent(src)}`);
}

const holes = (s) => (typeof s === "string" ? [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort() : []);

test("all languages have the same keys and placeholders as English", async () => {
  const { STRINGS, LANGUAGES } = await load();
  const en = STRINGS.en;
  for (const { code } of LANGUAGES) {
    const l = STRINGS[code];
    assert.ok(l, `missing language ${code}`);
    assert.deepEqual(Object.keys(l).sort(), Object.keys(en).sort(), `keys differ in ${code}`);
    for (const k of Object.keys(en)) {
      assert.deepEqual(holes(l[k]), holes(en[k]), `placeholders differ: ${code}.${k}`);
    }
    assert.equal(l.day_names.length, 7, `${code}.day_names`);
  }
});
