#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ -x "$HOME/.cargo/bin/cargo" ]; then
  PATH="$HOME/.cargo/bin:$PATH"
  export PATH
fi
nut_profile="${1:-release}"
case "$nut_profile" in
  release) cargo build --release --locked ;;
  debug) cargo build --locked ;;
  *) echo 'Usage: scripts/package-macos.sh [release|debug]' >&2; exit 1 ;;
esac
nut_app='dist/Weikav Studio.app'
mkdir -p "$nut_app/Contents/MacOS" "$nut_app/Contents/Resources"
cp "target/$nut_profile/weikav-studio" "$nut_app/Contents/MacOS/weikav-studio"
rm -f "$nut_app/Contents/Resources/protocol.md" "$nut_app/Contents/Resources/hardware-verification.md" "$nut_app/Contents/Resources/feature-parity.md" "$nut_app/Contents/Resources/bluetooth-investigation.md"
cp LICENSE README.md assets/SOURCES.md assets/Inter-LICENSE.txt "$nut_app/Contents/Resources/"
mkdir -p "$nut_app/Contents/Resources/docs/screenshots"
cp docs/screenshots/readme-keys.png docs/screenshots/readme-backlight.png "$nut_app/Contents/Resources/docs/screenshots/"
cp assets/fonts/NotoSansCJK-LICENSE.txt "$nut_app/Contents/Resources/NotoSansCJK-LICENSE.txt"
cp assets/app-icon/NUT75.icns "$nut_app/Contents/Resources/NUT75.icns"
cat > "$nut_app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Weikav Studio</string>
  <key>CFBundleDisplayName</key><string>Weikav Studio</string>
  <key>CFBundleIdentifier</key><string>studio.nut75.desktop</string>
  <key>CFBundleIconFile</key><string>NUT75.icns</string>
  <key>CFBundleExecutable</key><string>weikav-studio</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.9.4</string>
  <key>CFBundleVersion</key><string>91</string>
  <key>LSMinimumSystemVersion</key><string>10.15</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSHumanReadableCopyright</key><string>Copyright 2026 Weikav Studio contributors. WEIKAV assets belong to their respective owners.</string>
</dict></plist>
PLIST
xattr -cr "$nut_app"
codesign --force --deep --sign - "$nut_app"
echo "Built: $nut_app"
