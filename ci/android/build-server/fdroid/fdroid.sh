#!/usr/bin/env bash

# Updates the fdroid repo using the fdroid server tool.
# It requires a FDROID_REPO_DIR set up as following:
# FDROID_REPO_DIR/config.xml
# FDROID_REPO_DIR/metadata/net.mullvad.mullvadvpn.yml
# FDROID_REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/images/icon.png
# All these files are in same folder as this script.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
FDROID_REPO_DIR="$SCRIPT_DIR/fdroid"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
cd "$SCRIPT_DIR"

function main {
    if [[ -z ${YUBIKEY_PIN-} ]]; then
            echo "YUBIKEY_PIN pin must be set."
            exit 1
        fi

    if [[ $# -eq 0 ]]; then
        echo "Provide an apk to be moved into the repo"
        exit 1
    fi

    setup_repo "$1"

    YUBIKEY_PIN=$YUBIKEY_PIN \
    YUBIKEY_PATH=$(readlink -f /dev/android-jks-signing-key) \
    "$BUILD_DIR/android/scripts/containerized-sign.sh" "$FDROID_REPO_DIR" \
    'export JAVA_TOOL_OPTIONS="--add-opens=jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED" && fdroid update'
}

function setup_repo {
    if (( $# < 1 )); then
        echo "Provide the path to an apk file" >&2
        exit 1
    fi

    local apk="$1"

    local version_code=""
    # podman appends a trailing carriage return to the output. So we use `tr` to strip it
    version_code="$("$BUILD_DIR/building/container-run.sh" android "apkanalyzer manifest version-code /build/$apk" | tr -d "\r" || return 1)"

    # Copy the apk file into the repo
    cp "$BUILD_DIR/$apk" "$FDROID_REPO_DIR/repo/net.mullvad.mullvadvpn_$version_code.apk"

    # Copy the release notes into the repo
    mkdir -p "$FDROID_REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/changelogs"
    cp "$BUILD_DIR/android/src/main/play/release-notes/en-US/default.txt" \
    "$FDROID_REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/changelogs/${version_code}.txt"
}

main "$@"
