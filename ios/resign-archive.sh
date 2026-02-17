#!/usr/bin/env bash

# This script signs an existing unsigned .xcarchive and exports it as
# an IPA for distribution. It manually codesigns all binaries since
# xcodebuild -exportArchive cannot re-sign a fully unsigned archive.
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

SIGNING_IDENTITY="Apple Distribution: Mullvad VPN AB"
APP_BUNDLE_ID="net.mullvad.MullvadVPN"
EXTENSION_BUNDLE_ID="net.mullvad.MullvadVPN.PacketTunnel"

APP_DIR="$XCODE_ARCHIVE_DIR/Products/Applications/$PROJECT_NAME.app"
EXTENSION_DIR="$APP_DIR/PlugIns/PacketTunnel.appex"

if [[ ! -d "$IOS_PROVISIONING_PROFILES_DIR" ]]; then
    echo "Error: Provisioning profiles directory not found at $IOS_PROVISIONING_PROFILES_DIR"
    exit 1
fi

if [[ ! -d "$XCODE_ARCHIVE_DIR" ]]; then
    echo "Error: Archive not found at $XCODE_ARCHIVE_DIR"
    usage
fi

###########################################
# Locate provisioning profiles
###########################################

# Find the provisioning profile for a given bundle ID by inspecting the
# embedded application-identifier entitlement.
find_profile_for_bundle_id() {
    local target_bundle_id="$1"
    for profile_path in "$IOS_PROVISIONING_PROFILES_DIR"/*.mobileprovision; do
        local app_id
        app_id=$(security cms -D -i "$profile_path" 2>/dev/null \
            | plutil -extract Entitlements.application-identifier raw -o - -)
        # application-identifier is "TEAMID.bundle.id"
        if [[ "$app_id" == *".$target_bundle_id" ]]; then
            echo "$profile_path"
            return 0
        fi
    done
    echo "Error: No provisioning profile found for bundle ID $target_bundle_id" >&2
    exit 1
}

APP_PROFILE=$(find_profile_for_bundle_id "$APP_BUNDLE_ID")
EXTENSION_PROFILE=$(find_profile_for_bundle_id "$EXTENSION_BUNDLE_ID")

echo "App profile: $APP_PROFILE"
echo "Extension profile: $EXTENSION_PROFILE"

###########################################
# Resolve entitlements from project sources
###########################################

# Use the project's .entitlements files as the source of truth, not the
# provisioning profiles. Profiles may contain extra capabilities enabled
# on the App ID that the app does not use (e.g. url-filter-provider).

APP_ENTITLEMENTS_SRC="$SCRIPT_DIR/MullvadVPN/Supporting Files/MullvadVPN.entitlements"
EXTENSION_ENTITLEMENTS_SRC="$SCRIPT_DIR/PacketTunnel/PacketTunnel.entitlements"
SECURITY_GROUP_IDENTIFIER="group.net.mullvad.MullvadVPN"

resolve_entitlements() {
    local src="$1"
    local dest="$2"
    sed "s/\$(SECURITY_GROUP_IDENTIFIER)/$SECURITY_GROUP_IDENTIFIER/g" "$src" > "$dest"
}

APP_ENTITLEMENTS=$(mktemp)
EXTENSION_ENTITLEMENTS=$(mktemp)
trap 'rm -f "$APP_ENTITLEMENTS" "$EXTENSION_ENTITLEMENTS"' EXIT

resolve_entitlements "$APP_ENTITLEMENTS_SRC" "$APP_ENTITLEMENTS"
resolve_entitlements "$EXTENSION_ENTITLEMENTS_SRC" "$EXTENSION_ENTITLEMENTS"

echo ""
echo "App entitlements:"
cat "$APP_ENTITLEMENTS"
echo ""
echo "Extension entitlements:"
cat "$EXTENSION_ENTITLEMENTS"
echo ""

###########################################
# Embed provisioning profiles
###########################################

echo "Embedding provisioning profiles..."
cp "$APP_PROFILE" "$APP_DIR/embedded.mobileprovision"
cp "$EXTENSION_PROFILE" "$EXTENSION_DIR/embedded.mobileprovision"

###########################################
# Codesign
###########################################

# Sign frameworks first (no entitlements needed)
echo "Signing frameworks..."
for framework in "$APP_DIR"/Frameworks/*.framework; do
    echo "  $(basename "$framework")"
    codesign --force --sign "$SIGNING_IDENTITY" --timestamp "$framework"
done

# Sign the extension
echo "Signing PacketTunnel extension..."
codesign --force --sign "$SIGNING_IDENTITY" --timestamp \
    --entitlements "$EXTENSION_ENTITLEMENTS" \
    "$EXTENSION_DIR"

# Sign the main app
echo "Signing main app..."
codesign --force --sign "$SIGNING_IDENTITY" --timestamp \
    --entitlements "$APP_ENTITLEMENTS" \
    "$APP_DIR"

###########################################
# Package IPA
###########################################

echo ""
echo "Packaging IPA..."

IPA_STAGING=$(mktemp -d)
IPA_PATH="$BUILD_OUTPUT_DIR/$PROJECT_NAME.ipa"
trap 'rm -f "$APP_ENTITLEMENTS" "$EXTENSION_ENTITLEMENTS"; rm -rf "$IPA_STAGING"' EXIT

mkdir -p "$IPA_STAGING/Payload"
cp -a "$APP_DIR" "$IPA_STAGING/Payload/"
(cd "$IPA_STAGING" && zip -qr "$IPA_PATH" Payload)

echo ""
echo "Signed IPA exported to: $IPA_PATH"

# Verify signature
echo ""
echo "Verifying signature..."
codesign -dvv "$APP_DIR" 2>&1 | grep -E "^(Authority|TeamIdentifier|Identifier)"
