import test from "node:test";
import assert from "node:assert";
import crypto from "node:crypto";
import http from "node:http";
import { createApp, DEV_PUBLIC_KEY_HEX } from "../src/app.mjs";
import { JsonStore } from "../src/store.mjs";

async function setup() {
  let now = Date.parse("2026-10-15T10:00:00Z");
  const store = new JsonStore(null);
  store.putCode({ code: "DEV-LIFETIME", days: null });
  store.putCode({ code: "DEV-30D", days: 30 });
  const app = createApp({ store, clock: () => now, log: () => {} });
  const server = http.createServer(app.handle);
  await new Promise((r) => server.listen(0, r));
  const base = `http://127.0.0.1:${server.address().port}`;
  const rpc = async (fn, body) => {
    const res = await fetch(`${base}/rest/v1/rpc/${fn}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
    return { status: res.status, data: await res.json() };
  };
  return { store, rpc, close: () => server.close(), advance: (ms) => (now += ms), now: () => now };
}

const INSTALL = "a".repeat(32);
const DK = "1".repeat(64);
const DAY = 86_400_000;

/** Verifies a token the way the agent does and returns its claims. */
function verify(token) {
  const [p, s] = token.split(".");
  const payload = Buffer.from(p, "hex");
  const key = crypto.createPublicKey({ key: { kty: "OKP", crv: "Ed25519", x: Buffer.from(DEV_PUBLIC_KEY_HEX, "hex").toString("base64url") }, format: "jwk" });
  const msg = Buffer.concat([Buffer.from("htl-content-guard-license-v1\n"), payload]);
  assert.ok(crypto.verify(null, msg, key, Buffer.from(s, "hex")), "signature");
  return JSON.parse(payload.toString("utf8"));
}

test("register starts a signed trial bound to the device; it expires", async () => {
  const s = await setup();
  const r = await s.rpc("license_register", { p_install_id: INSTALL, p_device_key: DK, p_os: "windows" });
  assert.equal(r.status, 200);
  assert.equal(r.data.plan, "trial");
  const c = verify(r.data.token);
  assert.equal(c.id, INSTALL);
  assert.equal(c.dk, DK);
  assert.equal(c.plan, "trial");
  s.advance(DAY);
  const again = await s.rpc("license_register", { p_install_id: INSTALL, p_device_key: DK });
  assert.equal(again.data.trial_started, r.data.trial_started, "trial start must not reset");
  s.advance(4 * DAY);
  const expired = await s.rpc("license_check", { p_install_id: INSTALL, p_device_key: DK });
  assert.equal(verify(expired.data.token).plan, "expired");
  s.close();
});

test("another computer with the same Device ID gets no token", async () => {
  const s = await setup();
  await s.rpc("license_register", { p_install_id: INSTALL, p_device_key: DK });
  const other = await s.rpc("license_check", { p_install_id: INSTALL, p_device_key: "2".repeat(64) });
  assert.equal(other.status, 200);
  assert.equal(other.data.token, null);
  assert.equal(other.data.device_mismatch, true);
  s.close();
});

test("redeem codes; wrong guesses are limited", async () => {
  const s = await setup();
  await s.rpc("license_register", { p_install_id: INSTALL, p_device_key: DK });
  const life = await s.rpc("license_redeem", { p_install_id: INSTALL, p_device_key: DK, p_code: "dev-lifetime" });
  assert.equal(verify(life.data.token).plan, "premium");
  assert.equal(life.data.premium_until, null);
  // A used code cannot be redeemed again.
  const reuse = await s.rpc("license_redeem", { p_install_id: INSTALL, p_device_key: DK, p_code: "DEV-LIFETIME" });
  assert.equal(reuse.data.error, "invalid or used code");

  const other = "b".repeat(32);
  await s.rpc("license_register", { p_install_id: other, p_device_key: DK });
  for (let i = 0; i < 10; i++) await s.rpc("license_redeem", { p_install_id: other, p_device_key: DK, p_code: `GUESS-${i}` });
  const limited = await s.rpc("license_redeem", { p_install_id: other, p_device_key: DK, p_code: "DEV-30D" });
  assert.match(limited.data.error, /too many/);
  s.advance(3600_001);
  const timed = await s.rpc("license_redeem", { p_install_id: other, p_device_key: DK, p_code: "DEV-30D" });
  assert.equal(verify(timed.data.token).plan, "premium");
  assert.ok(timed.data.premium_until);
  s.close();
});

test("rejects bad input", async () => {
  const s = await setup();
  assert.equal((await s.rpc("license_register", { p_install_id: "short", p_device_key: DK })).status, 400);
  assert.equal((await s.rpc("license_register", { p_install_id: INSTALL, p_device_key: "nope" })).status, 400);
  assert.equal((await s.rpc("license_check", { p_install_id: "c".repeat(32), p_device_key: DK })).status, 404);
  assert.equal((await s.rpc("redeem_code", { p_install_id: INSTALL, p_code: "DEV-30D" })).status, 404);
  assert.equal((await s.rpc("nope", {})).status, 404);
  s.close();
});
