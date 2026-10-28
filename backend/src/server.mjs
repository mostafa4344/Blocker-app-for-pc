// Entry point for the local licensing mock: `npm start`.
import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createApp } from "./app.mjs";
import { JsonStore } from "./store.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const port = Number(process.env.PORT || 8787);
const dataDir = process.env.DATA_DIR || path.join(here, "..", "data");
const apiKey = process.env.API_KEY || null;

const store = new JsonStore(path.join(dataDir, "db.json"));
// Seed a couple of test activation codes for development.
if (!store.code("PREMIUM-LIFETIME")) store.putCode({ code: "PREMIUM-LIFETIME", days: null });
if (!store.code("PREMIUM-30D")) store.putCode({ code: "PREMIUM-30D", days: 30 });

const app = createApp({ store, apiKey });
// Dev only: serve test installers from DATA_DIR/files/ (stands in for the download host).
const handle = (req, res) => {
  const m = req.method === "GET" && req.url.match(/^\/files\/([\w.-]+)$/);
  if (!m) return app.handle(req, res);
  const file = path.join(dataDir, "files", m[1]);
  if (!fs.existsSync(file)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { "Content-Type": "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
};
http.createServer(handle).listen(port, () => console.log(`contentguard-license-mock on :${port} (data: ${dataDir})`));
