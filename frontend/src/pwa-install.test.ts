import { test } from "node:test";
import assert from "node:assert/strict";
import { installPromptState, installText } from "./pwa-install.ts";

test("mostra instalacao quando esta no celular, fora do app instalado e existe prompt nativo", () => {
  assert.deepEqual(installPromptState({
    dismissed: false,
    hasNativePrompt: true,
    isStandalone: false,
    mobile: true,
    platform: "android",
  }), {
    action: "native",
    visible: true,
  });
});

test("mostra instrucao iOS quando esta no celular sem prompt nativo", () => {
  assert.deepEqual(installPromptState({
    dismissed: false,
    hasNativePrompt: false,
    isStandalone: false,
    mobile: true,
    platform: "ios",
  }), {
    action: "ios-help",
    visible: true,
  });
});

test("nao mostra instalacao em desktop, app instalado ou banner dispensado", () => {
  assert.equal(installPromptState({
    dismissed: false,
    hasNativePrompt: true,
    isStandalone: false,
    mobile: false,
    platform: "desktop",
  }).visible, false);

  assert.equal(installPromptState({
    dismissed: false,
    hasNativePrompt: true,
    isStandalone: true,
    mobile: true,
    platform: "android",
  }).visible, false);

  assert.equal(installPromptState({
    dismissed: true,
    hasNativePrompt: true,
    isStandalone: false,
    mobile: true,
    platform: "android",
  }).visible, false);
});

test("textos de instalacao separam botao nativo de orientacao iOS", () => {
  assert.equal(installText("native").button, "Instalar app");
  assert.equal(installText("ios-help").button, "Ver instrução");
});
