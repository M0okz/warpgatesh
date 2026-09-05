"""Smoke-test the production relaunch handoff with disposable signed macOS bundles."""

import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tempfile
import time


def run_probe():
    if sys.platform != "darwin":
        raise SystemExit("This smoke test requires a macOS graphical session.")
    companion = Path(__file__).resolve().parents[1]
    binary = companion / "src-tauri/target/debug/examples/relaunch_probe"
    with tempfile.TemporaryDirectory(prefix="warpgatesh relaunch ' ") as directory:
        # Tauri deliberately refuses to relaunch executables reached through symlinks.
        root = Path(directory).resolve()
        for name in ("Probe.app", "Replacement.app"):
            bundle = root / name
            executable_name = "relaunch_probe" if name == "Probe.app" else "updated_probe"
            executable = bundle / "Contents/MacOS" / executable_name
            executable.parent.mkdir(parents=True)
            shutil.copy2(binary, executable)
            if name == "Replacement.app":
                data = executable.read_bytes()
                assert b"original build marker" in data
                executable.write_bytes(data.replace(b"original build marker", b"updated! build marker"))
            (bundle / "Contents/Info.plist").write_bytes(plistlib.dumps({
                "CFBundleExecutable": executable_name,
                "CFBundleIdentifier": "dev.warpgatesh.relaunch-probe",
                "CFBundleName": "WarpgateSH Relaunch Probe",
                "CFBundlePackageType": "APPL",
                "CFBundleShortVersionString": "0.1.14",
                "LSUIElement": True,
            }))
            subprocess.run([
                "/usr/bin/codesign", "--force", "--sign", "-", "--options", "runtime",
                "--timestamp=none", str(bundle),
            ], check=True, capture_output=True)

        # The fixture has its own bundle identifier and never loads user profiles or agents.
        with (root / "probe.log").open("w") as log:
            process = subprocess.Popen([
                "/usr/bin/open", "-n", "-W", str(root / "Probe.app"), "--args", str(root),
            ], stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 30
                while time.monotonic() < deadline and not (root / "reopened").exists():
                    time.sleep(0.1)
                assert (root / "started").exists(), "The initial bundle did not launch."
                assert (root / "installed").exists(), "The fixture did not replace the bundle."
                assert (root / "reopened").exists(), "The updated application did not reopen."
                result = json.loads((root / "reopened").read_text())
                assert result == {
                    "visible": True, "oldExited": True, "build": "updated! build marker",
                }, result
                process.wait(timeout=5)
                print("PASS: signed replacement reopened visibly after the old process exited; "
                      "renamed executable and paths with spaces/apostrophes supported.")
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=5)
                if not (root / "reopened").exists() and (root / "old-pid").exists():
                    pid = int((root / "old-pid").read_text())
                    try:
                        os.kill(pid, 15)
                    except ProcessLookupError:
                        pass


if __name__ == "__main__":
    run_probe()
