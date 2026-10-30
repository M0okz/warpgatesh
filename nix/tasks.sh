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
  cd apps/warpgatesh-companion
  npm ci
  npm run build
  npm run test:state
  npm run test:sidecars
  npm run test:updater-manifest
  npm run prepare:sidecars
  npm run prepare:native
  /usr/bin/xcrun swift test --package-path ../warpgatesh-macos
  /usr/bin/xcrun swift format lint --strict --recursive ../warpgatesh-macos/Sources ../warpgatesh-macos/Tests ../warpgatesh-macos/Package.swift
  cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
  cargo test --manifest-path src-tauri/Cargo.toml --locked
  cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings
  cargo build --manifest-path src-tauri/Cargo.toml --locked
  python3 ../warpgatesh-macos/scripts/test-bridge.py src-tauri/target/debug/warpgatesh-companion
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
    cd apps/warpgatesh-companion
    npm ci
    # Validate both architectures without requiring release credentials.
    export TAURI_ENV_TARGET_TRIPLE=universal-apple-darwin
    npm run tauri -- build --target universal-apple-darwin --bundles app,dmg --ci --no-sign "$@"
    npm run verify:bundle -- src-tauri/target/universal-apple-darwin/release/bundle/macos/WarpgateSH.app
    hdiutil verify "src-tauri/target/universal-apple-darwin/release/bundle/dmg/WarpgateSH_$(node -p 'require("./package.json").version')_universal.dmg"
    ;;
  release-macos)
    require_macos
    cd apps/warpgatesh-companion
    npm ci
    npm run bundle:macos:universal -- "$@"
    ;;
  build-site)
    cd apps/warpgatesh-site
    npm ci --ignore-scripts
    npm run build
    npm run check
    ;;
  *) echo "Unknown task: $task" >&2; exit 2 ;;
esac
