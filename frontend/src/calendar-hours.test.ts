import assert from "node:assert/strict";
import test from "node:test";
import { businessHours } from "./calendar-hours.ts";

test("gera a agenda das 09:00 as 22:00", () => {
  assert.deepEqual(businessHours(), [
    "09:00",
    "10:00",
    "11:00",
    "12:00",
    "13:00",
    "14:00",
    "15:00",
    "16:00",
    "17:00",
    "18:00",
    "19:00",
    "20:00",
    "21:00",
    "22:00",
  ]);
});
