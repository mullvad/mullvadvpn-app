#!/usr/bin/env bash
# shellcheck shell=bash
# Build universal installer for both ARM and x64.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"target"}

source scripts/utils/log

echo "Computing build version..."
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version)
log_header "Building universal Windows installer for Mullvad VPN $PRODUCT_VERSION"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --x64-installer)
            export WIN_X64_INSTALLER="$2"
            shift 2
            ;;
        --arm64-installer)
            export WIN_ARM64_INSTALLER="$2"
            shift 2
            ;;
        *)
            log_error "Unknown argument: $1"
            exit 1
            ;;
    esac
done

if [[ -z ${WIN_X64_INSTALLER-} ]] || [[ -z ${WIN_ARM64_INSTALLER-} ]]; then
    log_error "Must provide --x64-installer and --arm64-installer"
    exit 1
fi

cargo build --release -p windows-installer --target x86_64-pc-windows-msvc

cp "$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/release/windows-installer.exe" "dist/MullvadVPN-${PRODUCT_VERSION}.exe"

log_success "Built universal installer: dist/MullvadVPN-${PRODUCT_VERSION}.exe"
