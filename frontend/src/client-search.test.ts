import assert from "node:assert/strict";
import test from "node:test";
import { clientSearchMatches, clientDraftFromSearch } from "./client-search.ts";

const client = {
  id: 1,
  name: "Ana Souza",
  phone: "(11) 98888-7777",
  document: "123.456.789-00",
};

test("busca cliente por nome, telefone ou CPF", () => {
  assert.equal(clientSearchMatches(client, "ana"), true);
  assert.equal(clientSearchMatches(client, "988887777"), true);
  assert.equal(clientSearchMatches(client, "12345678900"), true);
  assert.equal(clientSearchMatches(client, "joao"), false);
});

test("preenche telefone ou CPF a partir da busca sem resultado", () => {
  assert.deepEqual(clientDraftFromSearch("11999998888"), { phone: "11999998888", document: "" });
  assert.deepEqual(clientDraftFromSearch("1133334444"), { phone: "", document: "" });
  assert.deepEqual(clientDraftFromSearch("123.456.789-00"), { phone: "", document: "12345678900" });
  assert.deepEqual(clientDraftFromSearch("Maria Cliente"), { phone: "", document: "" });
});
