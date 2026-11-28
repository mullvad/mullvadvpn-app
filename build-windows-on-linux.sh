#!/usr/bin/env bash
#
# Cross-compiles the Windows x64 installer of this app on a Linux host.
#
# Result: dist/MullvadVPN-<version>_x64.exe. The installer is NOT code signed.
#
# Needed on the host:
#   * Rust with the x86_64-pc-windows-msvc and i686-pc-windows-msvc targets
#   * cargo-xwin (`cargo install cargo-xwin`), clang, lld and llvm (lld-link, llvm-lib)
#   * protoc, git, curl, node and npm
#   * wine with 32-bit support (electron-builder runs the NSIS uninstaller generator under wine,
#     on Ubuntu/Debian that means the wine, wine64 and wine32:i386 packages)
#   * the dist-assets/binaries submodule: `git submodule update --init dist-assets/binaries`
#   * `npm ci` run once in desktop/
#
# cargo-xwin downloads the Microsoft CRT and Windows SDK. That requires accepting Microsoft's
# license, so this script only continues when you have set XWIN_ACCEPT_LICENSE=1 yourself.
#
# Optional environment:
#   RELAYS_JSON=<file>   Relay list to bundle. Default: downloaded from the Mullvad API.
#   FAST=1               No LTO and many codegen units. Much quicker to build, the binaries get
#                        somewhat larger and slower. The installer that came with this fork was built so.
#
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

: "${XWIN_ACCEPT_LICENSE:?Set XWIN_ACCEPT_LICENSE=1 to accept the Microsoft CRT/SDK license used by cargo-xwin}"
export XWIN_ARCH="${XWIN_ARCH:-x86,x86_64}"
# The same as build.sh does for Windows builds: embeds the application manifest in the daemon.
export MULLVAD_ADD_MANIFEST=1

if [[ "${FAST:-0}" == "1" ]]; then
    export CARGO_PROFILE_RELEASE_LTO=off
    export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
fi

if [[ ! -f dist-assets/binaries/x86_64-pc-windows-msvc/wireguard-nt/wireguard.dll ]]; then
    echo "dist-assets/binaries is empty. Run: git submodule update --init dist-assets/binaries" >&2
    exit 1
fi

release_dir="target/x86_64-pc-windows-msvc/release"

echo "==> Building the Windows binaries"
cargo xwin build --release --target x86_64-pc-windows-msvc \
    -p mullvad-daemon -p mullvad-cli -p mullvad-setup -p mullvad-problem-report -p windows-utils
# The NSIS plugin used by the installer is always a 32-bit DLL. installer.nsh loads it from
# target/i686-pc-windows-msvc/release.
cargo xwin build --release --target i686-pc-windows-msvc -p mullvad-nsis

echo "==> Placing the binaries where electron-builder looks for them"
for bin in mullvad-daemon mullvad mullvad-problem-report mullvad-setup; do
    cp "$release_dir/$bin.exe" dist-assets/
done
mkdir -p desktop/packages/windows-utils/dist/win32-x64-msvc
cp "$release_dir/windows_utils.dll" desktop/packages/windows-utils/dist/win32-x64-msvc/index.node

echo "==> Relay list"
relays=dist-assets/relays/relays.json
if [[ -n "${RELAYS_JSON:-}" ]]; then
    cp "$RELAYS_JSON" "$relays"
elif [[ ! -s "$relays" ]]; then
    curl -fsS https://api.mullvad.net/app/v1/relays -o "$relays"
fi
# Fails early on a truncated or non-JSON download.
node -e 'const r = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")); if (!r.wireguard?.relays?.length) process.exit(1)' "$relays"

echo "==> Building the GUI and the installer"
# Cargo can't link for Windows from a Linux host, so the windows-utils module is built above.
export SKIP_NATIVE_MODULES_BUILD=1
export CSC_IDENTITY_AUTO_DISCOVERY=false
export WINEDEBUG="${WINEDEBUG:--all}"
export WINEDLLOVERRIDES="${WINEDLLOVERRIDES:-mscoree,mshtml=d;winemenubuilder.exe=d}"
SOURCE_DATE_EPOCH="$(cat dist-assets/desktop-source-date-epoch.txt)"
export SOURCE_DATE_EPOCH
(cd desktop/packages/mullvad-vpn && npm run pack:win)

echo
echo "Done:"
ls -1 dist/MullvadVPN-*_x64.exe
