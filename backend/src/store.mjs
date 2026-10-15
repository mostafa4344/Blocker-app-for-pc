// JSON-file storage mirroring the Supabase `accounts` and `codes` tables.
import fs from "node:fs";
import path from "node:path";

export class JsonStore {
  constructor(file) {
    this.file = file;
    this.data = { accounts: {}, codes: {} };
    if (file && fs.existsSync(file)) this.data = { ...this.data, ...JSON.parse(fs.readFileSync(file, "utf8")) };
  }
  save() {
    if (!this.file) return;
    fs.mkdirSync(path.dirname(this.file), { recursive: true });
    const tmp = `${this.file}.tmp`;
    fs.writeFileSync(tmp, JSON.stringify(this.data, null, 2));
    fs.renameSync(tmp, this.file);
  }
  account(id) { return this.data.accounts[id] ?? null; }
  putAccount(a) { this.data.accounts[a.install_id] = a; this.save(); return a; }
  code(c) { return this.data.codes[c] ?? null; }
  putCode(c) { this.data.codes[c.code] = c; this.save(); }
}
