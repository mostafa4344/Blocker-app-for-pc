import test from "node:test";
import assert from "node:assert";
import http from "node:http";
import { createApp } from "../src/app.mjs";
import { JsonStore } from "../src/store.mjs";

async function setup() {
  let now = Date.parse("2026-10-15T10:00:00Z");
  const store = new JsonStore(null);
  store.putCode({ code: "PREMIUM-LIFETIME", days: null });
  store.putCode({ code: "PREMIUM-30D", days: 30 });
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
const DAY = 86_400_000;

test("register starts a trial; re-register keeps it; get_license expires it", async () => {
  const s = await setup();
  const r = await s.rpc("register_device", { p_install_id: INSTALL, p_trial_days: 4, p_os: "linux" });
  assert.equal(r.status, 200);
  assert.equal(r.data.plan, "trial");
  const started = r.data.trial_started;
  s.advance(DAY);
  const again = await s.rpc("register_device", { p_install_id: INSTALL });
  assert.equal(again.data.trial_started, started, "trial start must not reset");
  s.advance(4 * DAY);
  const expired = await s.rpc("get_license", { p_install_id: INSTALL });
  assert.equal(expired.data.plan, "expired");
  s.close();
});

test("redeem lifetime and timed codes", async () => {
  const s = await setup();
  await s.rpc("register_device", { p_install_id: INSTALL });
  const life = await s.rpc("redeem_code", { p_install_id: INSTALL, p_code: "premium-lifetime" });
  assert.equal(life.data.plan, "premium");
  assert.equal(life.data.premium_until, null);
  // A used code cannot be redeemed again.
  const reuse = await s.rpc("redeem_code", { p_install_id: INSTALL, p_code: "PREMIUM-LIFETIME" });
  assert.equal(reuse.status, 400);

  const other = "b".repeat(32);
  await s.rpc("register_device", { p_install_id: other });
  const timed = await s.rpc("redeem_code", { p_install_id: other, p_code: "PREMIUM-30D" });
  assert.equal(timed.data.plan, "premium");
  assert.ok(timed.data.premium_until);
  s.close();
});

test("rejects bad input", async () => {
  const s = await setup();
  assert.equal((await s.rpc("register_device", { p_install_id: "short" })).status, 400);
  assert.equal((await s.rpc("get_license", { p_install_id: "c".repeat(32) })).status, 404);
  assert.equal((await s.rpc("nope", {})).status, 404);
  s.close();
});
