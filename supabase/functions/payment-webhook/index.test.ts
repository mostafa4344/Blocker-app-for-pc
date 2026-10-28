// deno test supabase/functions/payment-webhook/index.test.ts
import { daysFor, hmacHex, safeEqual, verifyPaddle } from "./index.ts";

const assert = (c: boolean, m: string) => { if (!c) throw new Error(m); };

Deno.test("paddle signature: valid, tampered, stale, missing", async () => {
  const secret = "pdl_ntfset_test_secret";
  const body = JSON.stringify({ event_type: "transaction.completed", data: { custom_data: { install_id: "x".repeat(20) } } });
  const ts = 1_800_000_000;
  const h1 = await hmacHex(secret, `${ts}:${body}`);
  const header = `ts=${ts};h1=${h1}`;
  assert(await verifyPaddle(header, body, secret, ts + 10), "valid signature accepted");
  assert(!(await verifyPaddle(header, body + " ", secret, ts + 10)), "tampered body rejected");
  assert(!(await verifyPaddle(header, body, "other-secret", ts + 10)), "wrong secret rejected");
  assert(!(await verifyPaddle(header, body, secret, ts + 3600)), "old signature (replay) rejected");
  assert(!(await verifyPaddle(null, body, secret, ts)), "missing header rejected");
  assert(!(await verifyPaddle(header, body, "", ts)), "missing secret rejected");
});

Deno.test("products map to days; unknown is undefined, lifetime is null", () => {
  const t = { monthly: 30, lifetime: null } as Record<string, number | null>;
  assert(daysFor("monthly", t) === 30, "monthly");
  assert(daysFor("lifetime", t) === null, "lifetime");
  assert(daysFor("toString", t) === undefined, "prototype keys are not products");
  assert(daysFor("nope", t) === undefined, "unknown");
});

Deno.test("safeEqual", () => {
  assert(safeEqual("abc", "abc") && !safeEqual("abc", "abd") && !safeEqual("abc", "ab"), "compare");
});
