import assert from "node:assert/strict";
import test from "node:test";

import { createUpdaterManifest } from "./updater-manifest.mjs";

test("points both macOS architectures at the signed universal application", () => {
  const manifest = createUpdaterManifest({
    version: "0.1.7",
    notes: "Signed update",
    pubDate: "2026-08-13T12:00:00.000Z",
    url: "https://github.com/M0okz/warpgatesh/releases/download/v0.1.7/WarpgateSH.app.tar.gz",
    signature: "trusted signature",
  });

  assert.equal(manifest.version, "0.1.7");
  assert.deepEqual(manifest.platforms["darwin-aarch64"], manifest.platforms["darwin-x86_64"]);
  assert.equal(manifest.platforms["darwin-aarch64"].signature, "trusted signature");
});

test("refuses unsigned or insecure update metadata", () => {
  assert.throws(() =>
    createUpdaterManifest({
      version: "0.1.7",
      notes: "",
      pubDate: "2026-08-13T12:00:00.000Z",
      url: "http://example.test/update.tar.gz",
      signature: "",
    }),
  );
});
