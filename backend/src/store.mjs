// Storage layer. The JSON-file implementation mirrors the Supabase tables in
// supabase/migrations, so swapping it for a Supabase client only touches this file.
import fs from "node:fs";
import path from "node:path";

export class JsonStore {
  constructor(file) {
    this.file = file;
    this.data = { devices: [], requests: [], outbox: [] };
    if (file && fs.existsSync(file)) this.data = { ...this.data, ...JSON.parse(fs.readFileSync(file, "utf8")) };
  }

  save() {
    if (!this.file) return;
    fs.mkdirSync(path.dirname(this.file), { recursive: true });
    const tmp = `${this.file}.tmp`;
    fs.writeFileSync(tmp, JSON.stringify(this.data, null, 2));
    fs.renameSync(tmp, this.file);
  }

  // devices
  insertDevice(d) { this.data.devices.push(d); this.save(); return d; }
  getDevice(id) { return this.data.devices.find((d) => d.id === id) ?? null; }
  updateDevice(id, patch) {
    const d = this.getDevice(id);
    if (!d) return null;
    Object.assign(d, patch);
    this.save();
    return d;
  }
  listDevices() { return this.data.devices; }

  // requests (primary key: device_id + id)
  insertRequest(r) { this.data.requests.push(r); this.save(); return r; }
  getRequest(deviceId, id) { return this.data.requests.find((r) => r.device_id === deviceId && r.id === id) ?? null; }
  getRequestByTokenHash(hash) { return this.data.requests.find((r) => r.token_hash === hash) ?? null; }
  updateRequest(deviceId, id, patch) {
    const r = this.getRequest(deviceId, id);
    if (!r) return null;
    Object.assign(r, patch);
    this.save();
    return r;
  }

  // outbox (emails waiting to be sent)
  insertOutbox(m) { this.data.outbox.push(m); this.save(); return m; }
  listOutbox() { return this.data.outbox; }
}
