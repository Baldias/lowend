#!/bin/sh
# Builds target/lowend.app. The Info.plist mic description is required for macOS to grant microphone access.
set -e
cd "$(dirname "$0")/.."
cargo build --release
APP=target/lowend.app
rm -rf "$APP" && mkdir -p "$APP/Contents/MacOS"
cp target/release/lowend "$APP/Contents/MacOS/"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>lowend</string>
  <key>CFBundleIdentifier</key><string>io.github.baldias.lowend</string>
  <key>CFBundleExecutable</key><string>lowend</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSMicrophoneUsageDescription</key><string>lowend listens to your bass to tune it.</string>
</dict></plist>
PLIST
codesign --force --sign - "$APP" >/dev/null 2>&1 || true
echo "$APP"
