import test from "node:test";
import assert from "node:assert";
import http from "node:http";
import { createApp } from "../src/app.mjs";
import { JsonStore } from "../src/store.mjs";

async function setup() {
  let now = Date.parse("2026-10-14T10:00:00Z");
  const store = new JsonStore(null);
  const app = createApp({ store, publicUrl: "http://pub", clock: () => now, log: () => {} });
  const server = http.createServer(app.handle);
  await new Promise((r) => server.listen(0, r));
  const base = `http://127.0.0.1:${server.address().port}`;
  const call = async (method, path, body, secret, form) => {
    const headers = {};
    if (secret) headers.authorization = `Bearer ${secret}`;
    if (body !== undefined) headers["content-type"] = form ? "application/x-www-form-urlencoded" : "application/json";
    const res = await fetch(base + path, { method, headers, body: body === undefined ? undefined : form ? new URLSearchParams(body).toString() : JSON.stringify(body) });
    const text = await res.text();
    let data = text;
    try { data = JSON.parse(text); } catch { /* html */ }
    return { status: res.status, data };
  };
  return { store, app, call, close: () => server.close(), advance: (ms) => (now += ms) };
}

test("register, configure, request, partner approves", async () => {
  const s = await setup();
  const reg = await s.call("POST", "/v1/devices", { name: "Laptop", os: "windows" });
  assert.equal(reg.status, 201);
  const { device_id: id, device_secret: secret } = reg.data;

  assert.equal((await s.call("PUT", `/v1/devices/${id}`, { protection_level: "partner" })).status, 401);
  assert.equal((await s.call("PUT", `/v1/devices/${id}`, { protection_level: "partner", partner_email: "p@example.com", partner_name: "P", tamper_alerts: true }, secret)).status, 200);

  const r = await s.call("POST", `/v1/devices/${id}/requests`, { request_id: "r1", kind: { type: "uninstall" }, reason: "new PC", auto_approve_at: "2026-10-21T10:00:00Z" }, secret);
  assert.equal(r.status, 201);
  const mail = s.store.listOutbox().at(-1);
  assert.equal(mail.to, "p@example.com");
  const tok = mail.body.match(/approve\/([A-Za-z0-9_-]+)/)[1];

  const page = await s.call("GET", `/approve/${tok}`);
  assert.equal(page.status, 200);
  assert.match(page.data, /Laptop/);
  assert.equal((await s.call("POST", `/approve/${tok}`, { decision: "approve" }, null, true)).status, 200);
  assert.equal((await s.call("GET", `/v1/devices/${id}/requests/r1`, undefined, secret)).data.status, "approved");
  // The link cannot be reused.
  assert.equal((await s.call("GET", `/approve/${tok}`)).status, 410);
  s.close();
});

test("requests auto-approve after the deadline", async () => {
  const s = await setup();
  const { device_id: id, device_secret: secret } = (await s.call("POST", "/v1/devices", {})).data;
  await s.call("POST", `/v1/devices/${id}/requests`, { request_id: "r2", kind: { type: "change_protection" }, auto_approve_at: "2026-10-14T11:00:00Z" }, secret);
  assert.equal((await s.call("GET", `/v1/devices/${id}/requests/r2`, undefined, secret)).data.status, "pending");
  s.advance(61 * 60 * 1000);
  assert.equal((await s.call("GET", `/v1/devices/${id}/requests/r2`, undefined, secret)).data.status, "approved");
  s.close();
});

test("silent device alerts the partner once; approved uninstall does not", async () => {
  const s = await setup();
  const { device_id: id, device_secret: secret } = (await s.call("POST", "/v1/devices", { name: "Desk" })).data;
  await s.call("PUT", `/v1/devices/${id}`, { protection_level: "committed", partner_email: "p@example.com", tamper_alerts: true, language: "en" }, secret);
  await s.call("POST", `/v1/devices/${id}/heartbeat`, {}, secret);
  s.advance(10 * 60 * 1000);
  s.app.checkSilentDevices();
  assert.equal(s.store.listOutbox().length, 0);
  s.advance(25 * 60 * 1000);
  s.app.checkSilentDevices();
  s.app.checkSilentDevices();
  assert.equal(s.store.listOutbox().length, 1);
  assert.match(s.store.listOutbox()[0].subject, /stopped reporting/);

  await s.call("POST", `/v1/devices/${id}/heartbeat`, {}, secret);
  await s.call("POST", `/v1/devices/${id}/uninstalled`, {}, secret);
  s.advance(60 * 60 * 1000);
  s.app.checkSilentDevices();
  assert.equal(s.store.listOutbox().length, 2);
  assert.match(s.store.listOutbox()[1].subject, /uninstalled/);
  s.close();
});

test("rejects bad input", async () => {
  const s = await setup();
  const { device_id: id, device_secret: secret } = (await s.call("POST", "/v1/devices", {})).data;
  assert.equal((await s.call("POST", `/v1/devices/${id}/requests`, { request_id: "x", kind: { type: "nope" } }, secret)).status, 400);
  assert.equal((await s.call("GET", "/approve/not-a-token")).status, 410);
  assert.equal((await s.call("GET", `/v1/devices/${id}`, undefined, "wrong")).status, 401);
  s.close();
});
