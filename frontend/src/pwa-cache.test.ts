import assert from "node:assert/strict";
import test from "node:test";
import { isCacheableRequest, shouldCacheResponse } from "./pwa-cache.ts";

test("nao cacheia resposta 404 ou erro como se fosse asset valido", () => {
  assert.equal(shouldCacheResponse(new Response("ok", { status: 200 })), true);
  assert.equal(shouldCacheResponse(new Response("not found", { status: 404 })), false);
});

test("cacheia apenas GET do proprio dominio fora da api", () => {
  assert.equal(isCacheableRequest(new Request("https://example.test/app.js"), "https://example.test"), true);
  assert.equal(isCacheableRequest(new Request("https://example.test/api/clients"), "https://example.test"), false);
  assert.equal(isCacheableRequest(new Request("https://example.test/app.js", { method: "POST" }), "https://example.test"), false);
  assert.equal(isCacheableRequest(new Request("https://other.test/app.js"), "https://example.test"), false);
});
