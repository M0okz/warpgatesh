import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const companionDirectory = dirname(scriptDirectory);
const workspaceDirectory = dirname(dirname(companionDirectory));
const tag = process.argv[2]?.trim();

if (!tag || !/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag)) {
  throw new Error(`Expected a semantic release tag, received ${JSON.stringify(tag)}`);
}

const packageVersion = JSON.parse(
  readFileSync(join(companionDirectory, "package.json"), "utf8"),
).version;
const tauriVersion = JSON.parse(
  readFileSync(join(companionDirectory, "src-tauri", "tauri.conf.json"), "utf8"),
).version;
const workspaceManifest = readFileSync(
  join(workspaceDirectory, "Cargo.toml"),
  "utf8",
);
const cargoVersion = workspaceManifest.match(
  /\[workspace\.package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/,
)?.[1];
const tagVersion = tag.slice(1);

for (const [source, version] of [
  ["package.json", packageVersion],
  ["tauri.conf.json", tauriVersion],
  ["Cargo.toml", cargoVersion],
]) {
  if (version !== tagVersion) {
    throw new Error(`${source} declares ${version}; release tag declares ${tagVersion}`);
  }
}

console.log(`Release versions agree on ${tagVersion}`);
