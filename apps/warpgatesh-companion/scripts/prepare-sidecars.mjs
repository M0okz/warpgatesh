import { chmodSync, copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const companionDirectory = dirname(scriptDirectory);
const workspaceDirectory = dirname(dirname(companionDirectory));
const targetTriple =
  process.env.TAURI_ENV_TARGET_TRIPLE?.trim() || rustHostTriple();

if (!/^[a-zA-Z0-9_.-]+$/.test(targetTriple)) {
  throw new Error(`Invalid Rust target triple: ${targetTriple}`);
}

const cargo = process.env.CARGO?.trim() || "cargo";
const buildTargets =
  targetTriple === "universal-apple-darwin"
    ? ["aarch64-apple-darwin", "x86_64-apple-darwin"]
    : [targetTriple];

for (const buildTarget of buildTargets) {
  run(cargo, [
    "build",
    "--release",
    "--locked",
    "--target",
    buildTarget,
    "--package",
    "warpgatesh-cli",
    "--package",
    "warpgatesh-agent",
  ]);
}

const extension = process.platform === "win32" ? ".exe" : "";
const outputDirectory = join(companionDirectory, "src-tauri", "binaries");
mkdirSync(outputDirectory, { recursive: true });

for (const binary of ["warpgatesh", "warpgatesh-agent"]) {
  const destination = join(
    outputDirectory,
    `${binary}-${targetTriple}${extension}`,
  );
  const sources = buildTargets.map((buildTarget) =>
    join(
      workspaceDirectory,
      "target",
      buildTarget,
      "release",
      `${binary}${extension}`,
    ),
  );
  if (targetTriple === "universal-apple-darwin") {
    run("/usr/bin/lipo", ["-create", ...sources, "-output", destination]);
  } else {
    copyFileSync(sources[0], destination);
  }
  if (process.platform !== "win32") {
    chmodSync(destination, 0o755);
  }
  console.log(`Prepared ${binary} for ${targetTriple}`);
}

function rustHostTriple() {
  const result = spawnSync("rustc", ["--print", "host-tuple"], {
    cwd: workspaceDirectory,
    encoding: "utf8",
  });
  if (result.status !== 0 || !result.stdout.trim()) {
    throw new Error(
      `Could not determine the Rust host target: ${result.stderr.trim()}`,
    );
  }
  return result.stdout.trim();
}

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: workspaceDirectory,
    stdio: "inherit",
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}`);
  }
}
