import assert from "node:assert/strict";
import test from "node:test";
import { validateBarberRequiredFields } from "./barber-validation.ts";

test("CPF do profissional e obrigatorio e validado pelo digito verificador", () => {
  assert.equal(validateBarberRequiredFields({
    name: "Joao Barbeiro",
    email: "joao@teste.local",
    document: "",
    password: "TestPassword@123",
    isEditing: false,
  }), "Informe o CPF do profissional.");

  assert.equal(validateBarberRequiredFields({
    name: "Joao Barbeiro",
    email: "joao@teste.local",
    document: "123.456.789-00",
    password: "TestPassword@123",
    isEditing: false,
  }), "Informe um CPF válido para o profissional.");

  assert.equal(validateBarberRequiredFields({
    name: "Joao Barbeiro",
    email: "joao@teste.local",
    document: "123.456.789-09",
    password: "TestPassword@123",
    isEditing: false,
  }), "");
});

test("senha e obrigatoria somente no cadastro do profissional", () => {
  assert.equal(validateBarberRequiredFields({
    name: "Joao Barbeiro",
    email: "joao@teste.local",
    document: "123.456.789-09",
    password: "",
    isEditing: false,
  }), "Informe a senha do profissional.");

  assert.equal(validateBarberRequiredFields({
    name: "Joao Barbeiro",
    email: "joao@teste.local",
    document: "123.456.789-09",
    password: "",
    isEditing: true,
  }), "");
});
