#!/usr/bin/env bash

BIN_NAME=mullvad-vpn-cli
VERSION=2023.4
ARCHITECTURE=native
MAINTAINER="Mullvad VPN"

ARTIFACTS_DIR=target/release
ASSETS_DIR=dist-assets
BUILD_DIR=build

TARGET_BIN_DIR="/usr/bin/"
TARGET_RESOURCE_DIR="/opt/Mullvad VPN/resources/"

BINARIES=(
    "$ARTIFACTS_DIR/mullvad=$TARGET_BIN_DIR"
    "$ARTIFACTS_DIR/mullvad-daemon=$TARGET_BIN_DIR"
    "$ARTIFACTS_DIR/mullvad-exclude=$TARGET_BIN_DIR"
    "$ASSETS_DIR/linux/problem-report-link=$TARGET_BIN_DIR/mullvad-problem-report"
)
RESOURCES=(
    "$ARTIFACTS_DIR/mullvad-setup=$TARGET_RESOURCE_DIR"
    "$ARTIFACTS_DIR/mullvad-problem-report=$TARGET_RESOURCE_DIR"
    "$ARTIFACTS_DIR/libtalpid_openvpn_plugin.so=$TARGET_RESOURCE_DIR"
    "$ASSETS_DIR/binaries/x86_64-unknown-linux-gnu/openvpn=$TARGET_RESOURCE_DIR"
    "$ASSETS_DIR/ca.crt=$TARGET_RESOURCE_DIR"
    "$BUILD_DIR/relays.json=$TARGET_RESOURCE_DIR"
)
SYSTEM_SERVICES=(
    "$ASSETS_DIR/linux/mullvad-daemon.service=/usr/lib/systemd/system/"
    "$ASSETS_DIR/linux/mullvad-early-boot-blocking.service=/usr/lib/systemd/system/"
)
SHELL_COMPLETIONS=(
    "$BUILD_DIR/shell-completions/mullvad.bash=/usr/share/bash-completion/completions/mullvad"
    "$BUILD_DIR/shell-completions/_mullvad=/usr/share/zsh/site-functions/"
    "$BUILD_DIR/shell-completions/mullvad.fish=/usr/share/fish/vendor_completions.d/"
)

# Read arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift
            shift
            ;;
        --architecture)
            ARCHITECTURE="$2"
            shift
            shift
            ;;
        *)
            source scripts/utils/log
            log_error "Unknown parameter: $1"
            exit 1
            ;;
    esac
    shift
done

for package_type in rpm deb pacman; do
    fpm \
        -s dir \
        -t $package_type \
        --name $BIN_NAME \
        --package dist/ \
        --force \
        --license gpl3 \
        --version "$VERSION" \
        --architecture "$ARCHITECTURE" \
        --maintainer "$MAINTAINER" \
        --after-install="$ASSETS_DIR"/linux/after-install.sh \
        --after-remove="$ASSETS_DIR"/linux/after-remove.sh \
        --before-install="$ASSETS_DIR"/linux/before-install.sh \
        --before-remove="$ASSETS_DIR"/linux/before-remove.sh \
        "${BINARIES[@]}" "${RESOURCES[@]}" "${SYSTEM_SERVICES[@]}" "${SHELL_COMPLETIONS[@]}"
done
