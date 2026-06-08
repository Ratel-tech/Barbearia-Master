import assert from "node:assert/strict";
import test from "node:test";
import { accountLabel, loginPayload } from "./auth-account.ts";

test("monta login do estabelecimento com tipo de conta explicito", () => {
  assert.deepEqual(loginPayload(" ADMIN@Teste.Local ", " TestPassword@123 ", "establishment"), {
    email: "admin@teste.local",
    password: "TestPassword@123",
    account_type: "establishment",
  });
});

test("monta login do profissional com tipo de conta explicito", () => {
  assert.deepEqual(loginPayload(" barber@teste.local ", " TestPassword@123 ", "professional"), {
    email: "barber@teste.local",
    password: "TestPassword@123",
    account_type: "professional",
  });
});

test("nomeia tipos de acesso para a tela de login", () => {
  assert.equal(accountLabel("establishment"), "Estabelecimento");
  assert.equal(accountLabel("professional"), "Profissional");
});
