#!/usr/bin/env bash
# Builds the macOS app src-tauri/target/release/Photopea.app (see README.md for installing the tools).
#
#   ./build-macos.sh             first build on a computer (downloads Rust packages, needs internet)
#   ./build-macos.sh --offline   later builds, no internet needed
#
# If it says "Permission denied":  chmod +x build-macos.sh   (or run it as: bash build-macos.sh)
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
app="$root/src-tauri"
bin="$app/target/release/photopea-offline"
bundle="$app/target/release/Photopea.app"
offline=0
[ "${1:-}" = "--offline" ] && offline=1
started=$(date +%s)

step() { printf '\n\033[36m[%s/5] %s\033[0m\n' "$1" "$2"; }
ok()   { printf '      \033[32mOK\033[0m  %s\n' "$1"; }
fail() {
    printf '\n\033[31mBUILD FAILED: %s\033[0m\n' "$1"
    printf '\033[31mSee README.md (Building / Troubleshooting).\033[0m\n'
    exit 1
}

echo "Building Photopea Offline (macOS)"

# 1. tools
step 1 "Checking the build tools"
for tool in cargo rustc; do
    command -v "$tool" >/dev/null 2>&1 || fail "'$tool' was not found. Install Rust (README, Building on macOS, step 1c) and open a new Terminal window."
done
ok "$(rustc --version)"
ok "$(cargo --version)"
xcode-select -p >/dev/null 2>&1 || fail "the Xcode Command Line Tools are not installed. Run: xcode-select --install (README, Building on macOS, step 1b)."
ok "Command Line Tools: $(xcode-select -p)"

# 2. project files
step 2 "Checking the project files"
for f in src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/main.rs src-tauri/icons/icon.icns www/index.html www/code/pp/pp1792573386.js; do
    [ -f "$root/$f" ] || fail "missing $f. Copy the whole project folder again."
done
ok "source files present"

# 3. the program must not be running
step 3 "Checking that Photopea is not running"
if pgrep -x photopea-offline >/dev/null 2>&1; then fail "Photopea is running. Quit it (Cmd+Q) and run this script again."; fi
ok "not running"

# 4. compile, then wrap the program in a double-clickable Photopea.app
args=(build --release --locked)
[ "$offline" = 1 ] && args+=(--offline)
# keep this computer's paths (user name, folders) out of the program: rewrite them to neutral names
args+=(--config "build.rustflags=['--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}/registry=cargo','--remap-path-prefix=$app=src-tauri']")
step 4 "Compiling (cargo ${args[*]})"
if [ "$offline" = 1 ]; then
    echo "      Offline: using Rust packages already downloaded on this computer."
else
    echo "      The first build downloads Rust packages and takes 5-15 minutes; later builds take 1-3 minutes."
fi
(cd "$app" && cargo "${args[@]}") || fail "cargo failed (see the messages above)."
[ -f "$bin" ] || fail "the build finished but $bin was not created."
ok "compiled"
rm -rf "$bundle"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources" || fail "could not create $bundle."
cp "$bin" "$bundle/Contents/MacOS/photopea-offline"
cp "$app/icons/icon.icns" "$bundle/Contents/Resources/icon.icns"
cat > "$bundle/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Photopea</string>
  <key>CFBundleDisplayName</key><string>Photopea</string>
  <key>CFBundleIdentifier</key><string>local.photopea.offline</string>
  <key>CFBundleExecutable</key><string>photopea-offline</string>
  <key>CFBundleIconFile</key><string>icon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleVersion</key><string>1.0.0</string>
  <key>CFBundleShortVersionString</key><string>1.0.0</string>
  <key>LSMinimumSystemVersion</key><string>10.15</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST
ok "app bundle created"

# 5. result
step 5 "Done"
size=$(du -sm "$bundle" | cut -f1)
took=$(( $(date +%s) - started ))
echo
printf '\033[32mSaved to:  %s\033[0m\n' "$bundle"
printf 'Size:      %s MB    Took: %dm %02ds\n' "$size" $((took / 60)) $((took % 60))
echo "Fonts:     the fonts installed on this computer"
echo "Run it:    open \"$bundle\"   (or double-click it in Finder; drag it to Applications if you like)"
