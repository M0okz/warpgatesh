import { readFileSync, writeFileSync } from "node:fs";

import { createUpdaterManifest } from "./updater-manifest.mjs";

const [tag, url, signatureFile, notesFile, outputFile] = process.argv.slice(2);
if (!tag || !url || !signatureFile || !notesFile || !outputFile) {
  throw new Error(
    "Usage: generate-updater-manifest.mjs <tag> <artifact-url> <signature-file> <notes-file> <output-file>",
  );
}

const manifest = createUpdaterManifest({
  version: tag.replace(/^v/, ""),
  notes: readFileSync(notesFile, "utf8"),
  pubDate: new Date().toISOString(),
  url,
  signature: readFileSync(signatureFile, "utf8"),
});
writeFileSync(outputFile, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
console.log(`Generated signed updater manifest for ${manifest.version}`);
