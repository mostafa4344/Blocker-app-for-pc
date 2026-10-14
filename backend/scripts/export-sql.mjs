// Prints INSERT statements for moving data from the JSON store to Supabase.
// Usage: node scripts/export-sql.mjs [path/to/db.json] > seed.sql
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const file = process.argv[2] || path.join(here, "..", "data", "db.json");
const db = JSON.parse(fs.readFileSync(file, "utf8"));

const lit = (v) => {
  if (v === null || v === undefined) return "null";
  if (typeof v === "boolean") return v ? "true" : "false";
  if (typeof v === "number") return String(v);
  return `'${String(v).replace(/'/g, "''")}'`;
};

function inserts(table, rows, cols) {
  for (const r of rows) {
    const names = cols.map((c) => (c === "to" ? '"to"' : c)).join(", ");
    console.log(`insert into public.${table} (${names}) values (${cols.map((c) => lit(r[c])).join(", ")});`);
  }
}

console.log("begin;");
inserts("devices", db.devices ?? [], ["id", "secret_hash", "name", "os", "version", "protection_level", "tamper_alerts", "partner_name", "partner_email", "language", "last_heartbeat_at", "alert_sent_at", "uninstalled_at", "created_at"]);
inserts("requests", db.requests ?? [], ["device_id", "id", "kind", "reason", "auto_approve_at", "status", "token_hash", "decided_at", "created_at"]);
inserts("outbox", db.outbox ?? [], ["id", "to", "subject", "body", "created_at", "sent_at"]);
console.log("commit;");
