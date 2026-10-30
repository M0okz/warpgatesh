#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob
# Use an installed Apple toolchain compatible with Package.swift, without
# changing the runner's global xcode-select configuration.
for application in /Applications/Xcode.app /Applications/Xcode_*.app; do
  developer_dir="$application/Contents/Developer"
  [[ -d "$developer_dir" ]] || continue
  version=$(DEVELOPER_DIR="$developer_dir" /usr/bin/xcrun swift --version 2>/dev/null) || continue
  if [[ "$version" =~ Swift\ version\ ([0-9]+)\.([0-9]+) ]]; then
    major=${BASH_REMATCH[1]}
    minor=${BASH_REMATCH[2]}
    if (( major > 6 || (major == 6 && minor >= 2) )); then
      printf 'DEVELOPER_DIR=%s\n' "$developer_dir" >> "${GITHUB_ENV:?GitHub environment file is required}"
      printf 'Selected %s: %s\n' "$application" "$version"
      exit 0
    fi
  fi
done
printf 'The SwiftUI companion requires an installed Xcode with Swift 6.2 or later.\n' >&2
exit 1
