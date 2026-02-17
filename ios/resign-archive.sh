#!/usr/bin/env bash

# This script signs an existing .xcarchive and exports it as an IPA
# for distribution. It expects provisioning profiles and signing keys
# to be available on this machine.
set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

###########################################
# Parse arguments
###########################################

usage() {
    echo "Usage: $0 <provisioning-profiles-dir> [archive-path]"
    echo ""
    echo "Arguments:"
    echo "  provisioning-profiles-dir  Directory containing .mobileprovision files"
    echo "  archive-path               Path to .xcarchive (default: Build/MullvadVPN.xcarchive)"
    exit 1
}

if [[ $# -lt 1 ]]; then
    usage
fi

IOS_PROVISIONING_PROFILES_DIR="$1"

###########################################
# Build configuration
###########################################

PROJECT_NAME="MullvadVPN"
BUILD_OUTPUT_DIR="$SCRIPT_DIR/Build"
XCODE_ARCHIVE_DIR="${2:-$BUILD_OUTPUT_DIR/$PROJECT_NAME.xcarchive}"
EXPORT_OPTIONS_PATH="$SCRIPT_DIR/ExportOptions.plist"

if [[ ! -d "$IOS_PROVISIONING_PROFILES_DIR" ]]; then
    echo "Error: Provisioning profiles directory not found at $IOS_PROVISIONING_PROFILES_DIR"
    exit 1
fi

if [[ ! -d "$XCODE_ARCHIVE_DIR" ]]; then
    echo "Error: Archive not found at $XCODE_ARCHIVE_DIR"
    usage
fi

###########################################
# Install provisioning profiles
###########################################

SYSTEM_PROVISIONING_PROFILES_DIR="$HOME/Library/MobileDevice/Provisioning Profiles"

get_mobile_provisioning_uuid() {
  security cms -D -i "$1" | grep -aA1 UUID | grep -o "[-a-zA-Z0-9]\{36\}"
}

install_mobile_provisioning() {
    echo "Install system provisioning profiles into $SYSTEM_PROVISIONING_PROFILES_DIR"

    if [[ ! -d "$SYSTEM_PROVISIONING_PROFILES_DIR" ]]; then
        echo "Missing system provisioning profiles directory. Creating one."
        mkdir -p "$SYSTEM_PROVISIONING_PROFILES_DIR"
    fi

    for mobile_provisioning_path in "$IOS_PROVISIONING_PROFILES_DIR"/*.mobileprovision; do
        local profile_uuid
        profile_uuid=$(get_mobile_provisioning_uuid "$mobile_provisioning_path")
        local target_path="$SYSTEM_PROVISIONING_PROFILES_DIR/$profile_uuid.mobileprovision"

        if [[ -f "$target_path" ]]; then
            echo "Skip installing $mobile_provisioning_path"
        else
            echo "Install $mobile_provisioning_path -> $target_path"

            cp "$mobile_provisioning_path" "$target_path"
        fi
    done
}

install_mobile_provisioning

###########################################
# Sign and export IPA
###########################################

echo ""
echo "Signing archive: $XCODE_ARCHIVE_DIR"
echo ""

xcodebuild \
    -exportArchive \
    -archivePath "$XCODE_ARCHIVE_DIR" \
    -exportOptionsPlist "$EXPORT_OPTIONS_PATH" \
    -exportPath "$BUILD_OUTPUT_DIR" \
    -disableAutomaticPackageResolution

echo ""
echo "Signed IPA exported to: $BUILD_OUTPUT_DIR"
