#!/usr/bin/env bash
set -euo pipefail
: "${APPLE_SIGNING_IDENTITY:?A Developer ID signing identity is required}"
export TAURI_ENV_TARGET_TRIPLE=universal-apple-darwin
npm run tauri -- build --config src-tauri/tauri.release.conf.json --target universal-apple-darwin --no-bundle "$@"
# The native app entry point requires its backend to be signed already.
codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" src-tauri/target/universal-apple-darwin/release/warpgatesh-companion
npm run tauri -- bundle --config src-tauri/tauri.release.conf.json --target universal-apple-darwin --bundles app,dmg
npm run verify:bundle -- src-tauri/target/universal-apple-darwin/release/bundle/macos/WarpgateSH.app
