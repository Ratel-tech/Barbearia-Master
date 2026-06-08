import assert from "node:assert/strict";
import test from "node:test";
import { passwordResetPayload } from "./password-reset.ts";

test("monta payload de solicitacao de reset com email normalizado", () => {
  assert.deepEqual(passwordResetPayload("  admin@example.test  ", "establishment"), {
    email: "admin@example.test",
    account_type: "establishment",
  });
});

test("monta payload de confirmacao de reset com token e senha", () => {
  assert.deepEqual(passwordResetPayload(" codigo-123 ", " NovaSenha@123 "), {
    token: "codigo-123",
    password: "NovaSenha@123",
  });
});
