import assert from "node:assert/strict";
import test from "node:test";
import { apiErrorMessage, parseApiError } from "./api.ts";

test("extrai mensagem de erro JSON da API", () => {
  assert.equal(
    apiErrorMessage(400, '{"error":"dados invalidos: telefone ja cadastrado"}'),
    "dados invalidos: telefone ja cadastrado",
  );
});

test("mantem texto bruto quando resposta nao e JSON", () => {
  assert.equal(apiErrorMessage(500, "falha inesperada"), "falha inesperada");
});

test("detecta challenge de hcaptcha na resposta da API", () => {
  const error = parseApiError(429, JSON.stringify({
    error: "validacao humana necessaria",
    status: 429,
    challenge_required: true,
    captcha_provider: "hcaptcha",
    captcha_site_key: "10000000-ffff-ffff-ffff-000000000001",
    action: "login",
    retry_after_seconds: 900,
  }));

  assert.equal(error.challengeRequired, true);
  assert.equal(error.captchaProvider, "hcaptcha");
  assert.equal(error.captchaSiteKey, "10000000-ffff-ffff-ffff-000000000001");
  assert.equal(error.action, "login");
});
