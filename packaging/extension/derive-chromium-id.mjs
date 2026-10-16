// Derive the 32-character Chromium extension id from a signing key.
//
// The id is sha256(SubjectPublicKeyInfo DER)[:16], hex, with each hex nibble
// mapped 0-f -> a-p. This is exactly how Chrome/Edge/Brave compute the id, so
// the value printed here is the id a force-install policy must reference and
// the `appid` your update.xml must advertise.
//
// Usage:
//   node derive-chromium-id.mjs key.pem        # an RSA private OR public key (PEM)
//   node derive-chromium-id.mjs pubkey.der     # a DER SubjectPublicKeyInfo
//
// Generate a fresh signing key (keep key.pem SECRET — never commit it):
//   openssl genrsa 2048 | openssl pkcs8 -topk8 -nocrypt -out key.pem

import { createHash, createPublicKey } from "node:crypto";
import { readFileSync } from "node:fs";

const file = process.argv[2];
if (!file) {
  console.error("usage: node derive-chromium-id.mjs <key.pem|pubkey.der>");
  process.exit(2);
}

const raw = readFileSync(file);
// createPublicKey accepts a private key (and derives the public half) or a
// public key, in PEM or DER form.
let key;
try {
  key = createPublicKey(raw);
} catch {
  key = createPublicKey({ key: raw, format: "der", type: "spki" });
}

const spki = key.export({ type: "spki", format: "der" });
const hash = createHash("sha256").update(spki).digest();
const id = [...hash.subarray(0, 16)]
  .map((b) => b.toString(16).padStart(2, "0"))
  .join("")
  .split("")
  .map((c) => String.fromCharCode(97 + parseInt(c, 16)))
  .join("");

console.log(id);
