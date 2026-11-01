#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
task=${1:?Expected check, check-rust, check-companion, build-macos, release-macos or build-site}
shift

check_rust() {
  cargo fmt --all -- --check
  cargo test --workspace --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
}

check_companion() {
  cargo build -p warpgatesh-companion --locked
  python3 apps/warpgatesh-macos/scripts/swift.py test --package-path apps/warpgatesh-macos
  python3 apps/warpgatesh-macos/scripts/swift.py format lint --strict --recursive apps/warpgatesh-macos/Sources apps/warpgatesh-macos/Tests apps/warpgatesh-macos/Package.swift
  python3 apps/warpgatesh-macos/scripts/test-bridge.py target/debug/warpgatesh-companion
}

require_macos() {
  if [[ $(uname -s) != Darwin ]]; then
    echo 'The desktop bundle requires macOS and Xcode.' >&2
    exit 1
  fi
  /usr/bin/xcrun --find clang >/dev/null
}

case "$task" in
  check-rust) check_rust ;;
  check-companion) require_macos; check_companion ;;
  check)
    check_rust
    if [[ $(uname -s) == Darwin ]]; then check_companion; fi
    ;;
  build-macos)
    require_macos
    python3 apps/warpgatesh-macos/scripts/package.py "$@"
    ;;
  release-macos)
    require_macos
    python3 apps/warpgatesh-macos/scripts/release.py "$@"
    ;;
  build-site)
    cd apps/warpgatesh-site
    npm ci --ignore-scripts
    npm run build
    npm run check
    ;;
  *) echo "Unknown task: $task" >&2; exit 2 ;;
esac
