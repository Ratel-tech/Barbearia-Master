import assert from "node:assert/strict";
import test from "node:test";
import { registerAppServiceWorker } from "./pwa-registration.ts";

test("registra service worker como modulo e retorna true", async () => {
  let received = null as null | { url: string; type?: string };
  const ok = await registerAppServiceWorker(
    async (scriptURL, options) => {
      received = { url: scriptURL.toString(), type: options?.type };
    },
    new URL("https://example.test/service-worker.js"),
    () => {},
  );

  assert.equal(ok, true);
  assert.deepEqual(received, {
    url: "https://example.test/service-worker.js",
    type: "module",
  });
});

test("falha no registro retorna false e registra log", async () => {
  let message = "";
  const ok = await registerAppServiceWorker(
    async () => {
      throw new Error("boom");
    },
    new URL("https://example.test/service-worker.js"),
    (text) => {
      message = text;
    },
  );

  assert.equal(ok, false);
  assert.equal(message, "Falha ao registrar o service worker");
});
