#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
version=$(awk -F '"' '/^version =/ {print $2; exit}' Cargo.toml)
arch=$(uname -m)
case "$arch" in arm64) platform=AppleSilicon ;; x86_64) platform=Intel ;; *) echo "Unsupported architecture: $arch" >&2; exit 1 ;; esac
if [ "${1:-}" != --skip-build ]; then scripts/package-macos.sh release; fi
app='dist/NUT Studio.app'
[ -d "$app" ] || { echo 'Build the application first.' >&2; exit 1; }
staging=$(mktemp -d "${TMPDIR:-/tmp}/nut-studio-pkg.XXXXXX")
trap 'rm -rf "$staging"' EXIT HUP INT TERM
mkdir -p "$staging/root/Applications"
ditto "$app" "$staging/root/Applications/NUT Studio.app"
pkgbuild --analyze --root "$staging/root" "$staging/components.plist"
/usr/libexec/PlistBuddy -c 'Set :0:BundleIsRelocatable false' "$staging/components.plist"
mkdir -p dist/release
output="dist/release/NUT-Studio-$version-macOS-$platform.pkg"
pkgbuild --root "$staging/root" --component-plist "$staging/components.plist" --install-location / --identifier studio.nut75.desktop --version "$version" --ownership recommended "$output"
pkgutil --payload-files "$output" > "$staging/payload.txt"
[ -s "$staging/payload.txt" ]
echo "Built: $output"
