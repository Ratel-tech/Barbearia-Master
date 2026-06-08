import assert from "node:assert/strict";
import test from "node:test";
import { formatMobilePhoneInput, isBrazilianMobilePhone, isValidCpf, validateClientRequiredFields } from "./client-validation.ts";

test("aceita somente celular brasileiro com DDD e nono digito", () => {
  assert.equal(isBrazilianMobilePhone("(11) 99999-8888"), true);
  assert.equal(isBrazilianMobilePhone("11999998888"), true);
  assert.equal(isBrazilianMobilePhone("1133334444"), false);
  assert.equal(isBrazilianMobilePhone("11888887777"), false);
  assert.equal(isBrazilianMobilePhone("999998888"), false);
});

test("aceita placeholders repetidos quando cliente nao informa celular real", () => {
  assert.equal(isBrazilianMobilePhone("99999999"), true);
  assert.equal(isBrazilianMobilePhone("888"), true);
  assert.equal(isBrazilianMobilePhone("777"), true);
  for (const digit of "9876543210") {
    assert.equal(isBrazilianMobilePhone(digit.repeat(12)), true);
  }
});

test("nome, celular e frequencia de corte sao obrigatorios no cliente", () => {
  assert.equal(validateClientRequiredFields({ name: "Ana Souza", phone: "11999998888", haircut_frequency: "A cada 15 dias" }), "");
  assert.equal(validateClientRequiredFields({ name: "Ana Souza", phone: "11999998888", haircut_frequency: "" }), "Informe de quanto em quanto tempo o cliente corta o cabelo.");
  assert.equal(validateClientRequiredFields({ name: "Ana Souza", phone: "1133334444", haircut_frequency: "Mensal" }), "Informe um celular válido com DDD e 9 dígitos ou uma sequência repetida.");
});

test("valida CPF pelo algoritmo de digitos verificadores", () => {
  assert.equal(isValidCpf("123.456.789-09"), true);
  assert.equal(isValidCpf("111.444.777-35"), true);
  assert.equal(isValidCpf("123.456.789-00"), false);
  assert.equal(isValidCpf("111.111.111-11"), false);
  assert.equal(isValidCpf("123"), false);
});

test("CPF e opcional, mas quando informado deve ser valido", () => {
  assert.equal(validateClientRequiredFields({ name: "Ana Souza", phone: "11999998888", document: "", haircut_frequency: "Mensal" }), "");
  assert.equal(validateClientRequiredFields({ name: "Ana Souza", phone: "11999998888", document: "123.456.789-09", haircut_frequency: "Mensal" }), "");
  assert.equal(validateClientRequiredFields({ name: "Ana Souza", phone: "11999998888", document: "123.456.789-00", haircut_frequency: "Mensal" }), "Informe um CPF válido.");
});

test("formata celular limitando em 11 digitos", () => {
  assert.equal(formatMobilePhoneInput("119999988889999"), "(11) 99999-8888");
  assert.equal(formatMobilePhoneInput("11999998888"), "(11) 99999-8888");
  assert.equal(formatMobilePhoneInput("(11) 99999-8888"), "(11) 99999-8888");
  assert.equal(formatMobilePhoneInput("11999"), "(11) 999");
});

test("formata placeholder repetido com tres digitos antes do prefixo", () => {
  for (const digit of "9876543210") {
    assert.equal(formatMobilePhoneInput(digit.repeat(12)), `(${digit.repeat(3)}) ${digit.repeat(5)}-${digit.repeat(4)}`);
  }
});
