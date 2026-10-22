#!/usr/bin/env bash
# Builds the Linux program src-tauri/target/release/photopea-offline (see README.md for installing the tools).
#
#   ./build-linux.sh             first build on a computer (downloads Rust packages, needs internet)
#   ./build-linux.sh --offline   later builds, no internet needed
#
# If it says "Permission denied":  chmod +x build-linux.sh   (or run it as: bash build-linux.sh)
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
app="$root/src-tauri"
bin="$app/target/release/photopea-offline"
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

echo "Building Photopea Offline (Linux)"

# 1. tools
step 1 "Checking the build tools"
for tool in cargo rustc; do
    command -v "$tool" >/dev/null 2>&1 || fail "'$tool' was not found. Install Rust (README, Building on Linux, step 2) and open a new terminal."
done
ok "$(rustc --version)"
ok "$(cargo --version)"
command -v cc >/dev/null 2>&1 || fail "no C compiler ('cc') found. Install the system packages (README, Building on Linux, step 1)."
ok "C compiler: $(cc --version | head -n 1)"
command -v pkg-config >/dev/null 2>&1 || fail "'pkg-config' was not found. Install the system packages (README, Building on Linux, step 1)."
for lib in webkit2gtk-4.1 gtk+-3.0 libsoup-3.0; do
    pkg-config --exists "$lib" 2>/dev/null || fail "the library '$lib' was not found. Install the system packages (README, Building on Linux, step 1)."
done
ok "WebKitGTK $(pkg-config --modversion webkit2gtk-4.1), GTK $(pkg-config --modversion gtk+-3.0)"

# 2. project files
step 2 "Checking the project files"
for f in src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/main.rs www/index.html www/code/pp/pp1792573386.js; do
    [ -f "$root/$f" ] || fail "missing $f. Copy the whole project folder again."
done
ok "source files present"

# 3. the program must not be running
step 3 "Checking that Photopea is not running"
if pgrep -x photopea-offline >/dev/null 2>&1; then fail "Photopea is running. Close it and run this script again."; fi
ok "not running"

# 4. compile
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

# 5. result
step 5 "Done"
size=$(du -m "$bin" | cut -f1)
took=$(( $(date +%s) - started ))
echo
printf '\033[32mSaved to:  %s\033[0m\n' "$bin"
printf 'Size:      %s MB    Took: %dm %02ds\n' "$size" $((took / 60)) $((took % 60))
echo "Fonts:     the fonts installed on this computer"
echo "Run it:    \"$bin\""
