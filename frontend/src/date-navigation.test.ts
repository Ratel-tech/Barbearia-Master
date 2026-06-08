import assert from "node:assert/strict";
import test from "node:test";
import { addDays, buildMonthDays, formatDateKey } from "./date-navigation.ts";

test("navega entre dias mantendo chave local", () => {
  assert.equal(addDays("2026-05-28", -1), "2026-05-27");
  assert.equal(addDays("2026-05-28", 1), "2026-05-29");
});

test("monta calendario mensal com dias anteriores e subsequentes", () => {
  const days = buildMonthDays("2026-05-28", "2026-05-28");

  assert.equal(days.length, 42);
  assert.equal(days[0].date, "2026-04-26");
  assert.equal(days[5].date, "2026-05-01");
  assert.equal(days.some((day) => day.date === "2026-05-28" && day.today), true);
});

test("formata data local sem deslocar fuso", () => {
  assert.equal(formatDateKey(new Date(2026, 4, 28)), "2026-05-28");
});
