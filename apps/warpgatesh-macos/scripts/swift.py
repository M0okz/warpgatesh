#!/usr/bin/env python3
"""Run the selected Apple Swift toolchain without inherited Nix linker flags."""
import subprocess
import sys
from apple_toolchain import apple_environment

if __name__ == '__main__':
    raise SystemExit(subprocess.call(
        ['/usr/bin/xcrun', 'swift', *sys.argv[1:]], env=apple_environment(),
    ))
