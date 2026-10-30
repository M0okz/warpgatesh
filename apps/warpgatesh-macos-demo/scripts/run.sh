#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
python3 ../warpgatesh-macos/scripts/swift.py build
binary_dir="$(python3 ../warpgatesh-macos/scripts/swift.py build --show-bin-path)"
app_dir="$PWD/.build/WarpgateSH Demo.app"
mkdir -p "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources"
cp -R "$binary_dir/WarpgateSHDemo_WarpgateSHDemo.bundle" "$app_dir/Contents/Resources/"
cp "../warpgatesh-companion/src-tauri/icons/icon.icns" "$app_dir/Contents/Resources/AppIcon.icns"
cp "$binary_dir/WarpgateSHDemo" "$app_dir/Contents/MacOS/WarpgateSHDemo"
cat > "$app_dir/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>fr.warpgatesh.access-demo</string>
<key>CFBundleName</key><string>WarpgateSH Demo</string>
<key>CFBundleExecutable</key><string>WarpgateSHDemo</string>
<key>CFBundleIconFile</key><string>AppIcon</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>LSMinimumSystemVersion</key><string>13.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
open "$app_dir"
