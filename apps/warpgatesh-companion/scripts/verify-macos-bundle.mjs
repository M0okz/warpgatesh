import { accessSync, constants, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const companionDirectory = dirname(scriptDirectory);
const packageMetadata = JSON.parse(
  readFileSync(join(companionDirectory, "package.json"), "utf8"),
);
const bundle =
  process.argv[2] ||
  join(
    companionDirectory,
    "src-tauri",
    "target",
    "release",
    "bundle",
    "macos",
    "WarpgateSH.app",
  );
const executables = join(bundle, "Contents", "MacOS");
const companion = join(executables, "warpgatesh-companion");
const cli = join(executables, "warpgatesh");
const agent = join(executables, "warpgatesh-agent");

for (const executable of [companion, cli, agent]) {
  accessSync(executable, constants.R_OK | constants.X_OK);
}

if (bundle.includes("universal-apple-darwin")) {
  for (const executable of [companion, cli, agent]) {
    const architectures = capture("/usr/bin/lipo", ["-archs", executable])
      .split(/\s+/)
      .sort();
    if (architectures.join(" ") !== "arm64 x86_64") {
      throw new Error(
        `Unexpected architectures for ${executable}: ${architectures.join(" ")}`,
      );
    }
  }
}

const cliVersion = capture(cli, ["--version"]);
if (cliVersion !== `warpgatesh ${packageMetadata.version}`) {
  throw new Error(
    `Unexpected bundled CLI version: ${JSON.stringify(cliVersion)}`,
  );
}

const agentHelp = capture(agent, ["--help"]);
if (agentHelp !== "Usage: warpgatesh-agent [--once]") {
  throw new Error(
    `Unexpected bundled agent output: ${JSON.stringify(agentHelp)}`,
  );
}

console.log(`Verified WarpgateSH ${packageMetadata.version} bundle at ${bundle}`);

function capture(command, args) {
  const result = spawnSync(command, args, { encoding: "utf8" });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(
      `${command} exited with status ${result.status}: ${result.stderr.trim()}`,
    );
  }
  return result.stdout.trim();
}
