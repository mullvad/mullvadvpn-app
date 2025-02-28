#!/usr/bin/env bash

# This script is used to build, and optionally sign the downloader, always in release mode.

# This script performs the equivalent of the following profile:
#
# [profile.release]
# strip = true
# opt-level = 'z'
# codegen-units = 1
# lto = true
# panic = 'abort'
#
# We cannot set all of the above directly in Cargo.toml since some must be set for the entire
# workspace.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# shellcheck disable=SC1091
source ../scripts/utils/host
# shellcheck disable=SC1091
source ../scripts/utils/log

CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"../target"}
DIST_DIR="./dist"

function build_executable {
    local -a target_args=()

    if [[ -n "${1-}" ]]; then
        target_args+=(--target "$1")
    fi

    # Old bash versions complain about empty array expansion when -u is set
    set +u

    RUSTFLAGS="-C codegen-units=1 -C panic=abort -C strip=symbols -C opt-level=z" \
        cargo build --bin installer-downloader --release "${target_args[@]}"

    set -u
}

function dist_windows_app {
    cp "$CARGO_TARGET_DIR/release/installer-downloader.exe" "$DIST_DIR/MullvadDownloader.exe"
}

# Combine executables on macOS
function lipo_executables {
    local target_exes
    target_exes=()

    rm -rf "$DIST_DIR/installer-downloader"

    case $HOST in
        x86_64-apple-darwin) target_exes=(
            "$CARGO_TARGET_DIR/release/installer-downloader"
            "$CARGO_TARGET_DIR/aarch64-apple-darwin/release/installer-downloader"
        )
        ;;
        aarch64-apple-darwin) target_exes=(
            "$CARGO_TARGET_DIR/release/installer-downloader"
            "$CARGO_TARGET_DIR/x86_64-apple-darwin/release/installer-downloader"
        )
        ;;
    esac

    lipo "${target_exes[@]}" -create -output "$DIST_DIR/installer-downloader"
}

function dist_macos_app {
    local app_path
    bundle_name="MullvadDownloader"
    bundle_id="net.mullvad.$bundle_name"
    app_path="$DIST_DIR/$bundle_name.app/"

    # Build app bundle
    echo "Building $app_path..."

    rm -rf "$app_path"

    mkdir -p "$app_path/Contents/Resources"
    cp "../dist-assets/icon.icns" "$app_path/Contents/Resources/"

    mkdir -p "$app_path/Contents/MacOS"

    cp ./assets/Info.plist "$app_path/Contents/Info.plist"
    cp "$DIST_DIR/installer-downloader" "$app_path/Contents/MacOS/installer-downloader"

    # Ad-hoc sign app bundle
    codesign --force --deep --identifier "$bundle_id" --sign - \
        --timestamp=none --verbose=0 -o runtime \
        "$DIST_DIR/$bundle_name.app"

    # Pack in .dmg
    echo "Creating .dmg image..."

    hdiutil create -volname "MullvadDownloader" -srcfolder "$app_path" -ov -format UDZO \
        "$DIST_DIR/MullvadDownloader.dmg"

    # TODO: sign image?
}

mkdir -p "$DIST_DIR"

if [[ "$(uname -s)" == "Darwin" ]]; then
    case $HOST in
        x86_64-apple-darwin) TARGETS=("" aarch64-apple-darwin);;
        aarch64-apple-darwin) TARGETS=("" x86_64-apple-darwin);;
    esac

    for t in "${TARGETS[@]:-"$HOST"}"; do
        build_executable "$t"
    done

    lipo_executables
    dist_macos_app
elif [[ "$(uname -s)" == "MINGW"* ]]; then
    build_executable
    dist_windows_app
fi
