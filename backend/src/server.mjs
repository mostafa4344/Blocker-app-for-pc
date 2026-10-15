// Entry point for the local licensing mock: `npm start`.
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
http.createServer(app.handle).listen(port, () => console.log(`contentguard-license-mock on :${port} (data: ${dataDir})`));
