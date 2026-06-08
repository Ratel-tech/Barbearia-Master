import assert from "node:assert/strict";
import test from "node:test";
import { checkoutPaymentSummary } from "./checkout-payments.ts";

test("soma formas de pagamento e transforma excedente em gorjeta", () => {
  assert.deepEqual(
    checkoutPaymentSummary(10_000, [
      { method: "cash", amount: "50" },
      { method: "pix", amount: "70" },
    ]),
    { paid_cents: 12_000, tip_cents: 2_000, remaining_cents: 0 },
  );
});

test("calcula valor restante quando pagamento fica abaixo do subtotal", () => {
  assert.deepEqual(
    checkoutPaymentSummary(10_000, [{ method: "cash", amount: "80" }]),
    { paid_cents: 8_000, tip_cents: 0, remaining_cents: 2_000 },
  );
});

