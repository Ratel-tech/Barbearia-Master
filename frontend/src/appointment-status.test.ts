import assert from "node:assert/strict";
import test from "node:test";
import { appointmentStatusClass, canCheckoutAppointment, canEditAppointment } from "./appointment-status.ts";

test("mapeia status do agendamento para cor visual", () => {
  assert.equal(appointmentStatusClass("scheduled"), "is-waiting");
  assert.equal(appointmentStatusClass("waiting"), "is-waiting");
  assert.equal(appointmentStatusClass("in_chair"), "is-active");
  assert.equal(appointmentStatusClass("completed"), "is-completed");
  assert.equal(appointmentStatusClass("cancelled"), "is-cancelled");
});

test("bloqueia checkout de agendamento concluido ou cancelado", () => {
  assert.equal(canCheckoutAppointment("scheduled"), true);
  assert.equal(canCheckoutAppointment("waiting"), true);
  assert.equal(canCheckoutAppointment("in_chair"), true);
  assert.equal(canCheckoutAppointment("completed"), false);
  assert.equal(canCheckoutAppointment("cancelled"), false);
});

test("bloqueia edicao de agendamento concluido", () => {
  assert.equal(canEditAppointment("scheduled"), true);
  assert.equal(canEditAppointment("waiting"), true);
  assert.equal(canEditAppointment("in_chair"), true);
  assert.equal(canEditAppointment("cancelled"), true);
  assert.equal(canEditAppointment("completed"), false);
});
