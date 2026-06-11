import { test } from "node:test";
import assert from "node:assert/strict";
import { detectInstallPlatform, installPromptState, installText } from "./pwa-install.ts";

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

test("detecta iPadOS em modo desktop como iOS", () => {
  assert.equal(detectInstallPlatform({
    maxTouchPoints: 5,
    navigatorPlatform: "MacIntel",
    userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
  }), "ios");
});
