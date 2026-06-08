import assert from "node:assert/strict";
import test from "node:test";
import type { Appointment, Commission } from "./api.ts";
import { commissionSummary, professionalAgendaSummary } from "./professional-portal.ts";

const baseAppointment: Appointment = {
  id: 1,
  client_id: 1,
  client_name: "Cliente",
  barber_id: 7,
  barber_name: "Profissional",
  starts_at: "2026-06-08T09:00",
  status: "scheduled",
  total_cents: 5000,
  services: "Corte",
  service_ids: "1",
};

test("resume agenda do profissional para o dia atual", () => {
  const summary = professionalAgendaSummary([
    { ...baseAppointment, id: 1, starts_at: "2026-06-08T09:00", status: "scheduled" },
    { ...baseAppointment, id: 2, starts_at: "2026-06-08T10:00", status: "in_chair" },
    { ...baseAppointment, id: 3, starts_at: "2026-06-08T11:00", status: "completed" },
    { ...baseAppointment, id: 4, starts_at: "2026-06-09T09:00", status: "scheduled" },
  ], "2026-06-08");

  assert.equal(summary.today.length, 3);
  assert.equal(summary.openCount, 2);
  assert.equal(summary.completedCount, 1);
  assert.equal(summary.nextAppointment?.id, 1);
});

test("resume comissoes configuradas do profissional", () => {
  const commissions: Commission[] = [
    { barber_id: 7, service_id: 1, service_name: "Corte", price_cents: 5000, commission_percent: 40, estimated_return_cents: 2000 },
    { barber_id: 7, service_id: 2, service_name: "Barba", price_cents: 3000, commission_percent: 50, estimated_return_cents: 1500 },
  ];

  assert.deepEqual(commissionSummary(commissions), {
    services: 2,
    estimated_return_cents: 3500,
    average_percent: 45,
  });
});
