#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck disable=SC1091
source "$HOME/.cargo/env"

cargo test
cargo build --release

APP="DevCache.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp target/release/DevCache "$APP/Contents/MacOS/DevCache"
cat > "$APP/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>DevCache</string>
  <key>CFBundleIdentifier</key>
  <string>app.devcache.mac</string>
  <key>CFBundleName</key>
  <string>DevCache</string>
  <key>CFBundleDisplayName</key>
  <string>개발 캐시</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.utilities</string>
</dict>
</plist>
EOF
codesign --force --sign - "$APP"
open "$APP"
