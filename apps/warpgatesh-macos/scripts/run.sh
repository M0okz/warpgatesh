#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
swift build
binary_dir="$(swift build --show-bin-path)"
repo_dir="$(cd ../.. && pwd)"
cargo build --manifest-path "$repo_dir/apps/warpgatesh-companion/src-tauri/Cargo.toml" --locked
cargo build --manifest-path "$repo_dir/Cargo.toml" --locked --package warpgatesh-cli --package warpgatesh-agent
app_dir="$PWD/.build/WarpgateSH Preview.app"
mkdir -p "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources"
cp "$binary_dir/warpgatesh-ui" "$app_dir/Contents/MacOS/warpgatesh-ui"
cp "$repo_dir/apps/warpgatesh-companion/src-tauri/target/debug/warpgatesh-companion" "$app_dir/Contents/MacOS/warpgatesh-companion"
cp "$repo_dir/target/debug/warpgatesh" "$repo_dir/target/debug/warpgatesh-agent" "$app_dir/Contents/MacOS/"
cp -R "$binary_dir/WarpgateSH_WarpgateSH.bundle" "$app_dir/Contents/Resources/"
cp "../warpgatesh-companion/src-tauri/icons/icon.icns" "$app_dir/Contents/Resources/icon.icns"
cat > "$app_dir/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>dev.warpgatesh.preview</string>
<key>CFBundleName</key><string>WarpgateSH Preview</string>
<key>CFBundleExecutable</key><string>warpgatesh-ui</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleIconFile</key><string>icon.icns</string>
<key>LSMinimumSystemVersion</key><string>13.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
# The default preview is isolated: no backend, profiles, tokens, agent or network.
if [ "${1:-}" = "--live" ]; then
  open "$app_dir"
else
  open "$app_dir" --args --demo
fi
