export type CheckoutPaymentDraft = {
  method: string;
  amount: string;
};

export function amountToCents(value: string) {
  const amount = Number(value.replace(",", "."));
  if (!Number.isFinite(amount) || amount <= 0) return 0;
  return Math.round(amount * 100);
}

export function checkoutPaymentSummary(subtotal_cents: number, payments: CheckoutPaymentDraft[]) {
  const paid_cents = payments.reduce((sum, payment) => sum + amountToCents(payment.amount), 0);
  return {
    paid_cents,
    tip_cents: Math.max(paid_cents - subtotal_cents, 0),
    remaining_cents: Math.max(subtotal_cents - paid_cents, 0),
  };
}

