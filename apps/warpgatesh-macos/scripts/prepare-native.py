#!/usr/bin/env python3
"""Build native SwiftUI sidecars before the existing signed Tauri packaging step."""
import os
import pathlib
import shutil
import subprocess
from apple_toolchain import apple_environment

root = pathlib.Path(__file__).resolve().parents[1]
companion = root.parent / "warpgatesh-companion" / "src-tauri"
triple = os.environ.get("TAURI_ENV_TARGET_TRIPLE", "").strip()
if not triple:
    triple = subprocess.check_output(["rustc", "--print", "host-tuple"], text=True).strip()
architectures = {"universal-apple-darwin": ["arm64", "x86_64"], "aarch64-apple-darwin": ["arm64"], "x86_64-apple-darwin": ["x86_64"]}
if triple not in architectures:
    raise SystemExit("The native companion requires a macOS target")
args = ["/usr/bin/xcrun", "swift", "build", "--package-path", str(root), "--configuration", "release"]
for arch in architectures[triple]:
    args.extend(["--arch", arch])
apple_env = apple_environment()
subprocess.run(args, check=True, env=apple_env)
binaries = pathlib.Path(subprocess.check_output(args + ["--show-bin-path"], text=True, env=apple_env).strip())
output = companion / "binaries"
output.mkdir(exist_ok=True)
shutil.copy2(binaries / "warpgatesh-ui", output / ("warpgatesh-ui-" + triple))
if triple == "universal-apple-darwin":
    for arch, rust_target in [("arm64", "aarch64-apple-darwin"), ("x86_64", "x86_64-apple-darwin")]:
        subprocess.run(["/usr/bin/lipo", str(binaries / "warpgatesh-ui"), "-thin", arch, "-output", str(output / ("warpgatesh-ui-" + rust_target))], check=True)
resources = companion / "native-resources"
resources.mkdir(exist_ok=True)
shutil.copytree(binaries / "WarpgateSH_WarpgateSH.bundle", resources / "WarpgateSH_WarpgateSH.bundle", dirs_exist_ok=True)
