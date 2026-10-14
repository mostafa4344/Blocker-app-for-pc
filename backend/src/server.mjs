// Entry point: `npm start` (PORT, DATA_DIR and PUBLIC_URL are configurable).
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createApp } from "./app.mjs";
import { JsonStore } from "./store.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const port = Number(process.env.PORT || 8787);
const dataDir = process.env.DATA_DIR || path.join(here, "..", "data");
const publicUrl = process.env.PUBLIC_URL || `http://localhost:${port}`;

const store = new JsonStore(path.join(dataDir, "db.json"));
const app = createApp({ store, publicUrl });
http.createServer(app.handle).listen(port, () => console.log(`focusfort-backend on :${port} (data: ${dataDir})`));
setInterval(app.checkSilentDevices, 60 * 1000);
