#!/usr/bin/env python3
"""Read-only macOS check after closing or reopening the native companion window."""
import argparse
import re
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--bundle-id", default="dev.warpgatesh.preview")
parser.add_argument("--expect", choices=["menu-bar", "window"], required=True)
args = parser.parse_args()
found = subprocess.check_output(
    ["/usr/bin/lsappinfo", "find", "bundleid=" + args.bundle_id], text=True
)
application = re.search(r"ASN:0x[0-9a-f]+-0x[0-9a-f]+", found)
if application is None:
    parser.exit(1, "FAIL: the companion is no longer running.\n")
info = subprocess.check_output(
    ["/usr/bin/lsappinfo", "info", application.group()], text=True
)
kind = re.search(r'type="([^"]+)"', info)
expected = "UIElement" if args.expect == "menu-bar" else "Foreground"
if kind is None or kind.group(1) != expected:
    parser.exit(1, f"FAIL: expected {expected} application mode.\n")
print(f"PASS: companion is running in {args.expect} mode.")
