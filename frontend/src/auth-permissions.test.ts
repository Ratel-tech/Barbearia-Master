import { strict as assert } from "node:assert";
import test from "node:test";
import { canAccessPage, pagesForRole } from "./auth-permissions.ts";

test("profissional acessa somente agenda e comissoes", () => {
  assert.deepEqual(pagesForRole("barber"), ["agenda", "comissoes"]);
  assert.equal(canAccessPage("barber", "agenda"), true);
  assert.equal(canAccessPage("barber", "comissoes"), true);
  assert.equal(canAccessPage("barber", "financeiro"), false);
  assert.equal(canAccessPage("barber", "profissionais"), false);
});

test("dono acessa o sistema administrativo completo", () => {
  assert.equal(canAccessPage("owner", "financeiro"), true);
  assert.equal(canAccessPage("owner", "profissionais"), true);
  assert.equal(canAccessPage("owner", "comissoes"), false);
});
