import assert from "node:assert/strict";
import test from "node:test";
import { appointmentServices, serviceStatusLabel } from "./service-catalog.ts";

const services = [
  { id: 1, name: "Corte", description: "", duration_minutes: 45, price_cents: 8000, category: "cabelo", active: true },
  { id: 2, name: "Barba antiga", description: "", duration_minutes: 30, price_cents: 5000, category: "barba", active: false },
];

test("agenda oferece somente servicos ativos", () => {
  assert.deepEqual(appointmentServices(services).map((service) => service.id), [1]);
});

test("catalogo identifica servicos ativos e inativos", () => {
  assert.equal(serviceStatusLabel(true), "Ativo");
  assert.equal(serviceStatusLabel(false), "Inativo");
});
