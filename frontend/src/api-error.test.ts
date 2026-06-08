import assert from "node:assert/strict";
import test from "node:test";
import { apiErrorMessage } from "./api.ts";

test("extrai mensagem de erro JSON da API", () => {
  assert.equal(
    apiErrorMessage(400, '{"error":"dados invalidos: telefone ja cadastrado"}'),
    "dados invalidos: telefone ja cadastrado",
  );
});

test("mantem texto bruto quando resposta nao e JSON", () => {
  assert.equal(apiErrorMessage(500, "falha inesperada"), "falha inesperada");
});
