#!/usr/bin/env bash

# This script is used to build and ship the iOS app
set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

###########################################
# Verify environment configuration
###########################################

if [[ -z ${IOS_APPLE_ID-} ]]; then
    echo "The variable IOS_APPLE_ID is not set."
    exit
fi

if [[ -z ${IOS_APPLE_ID_PASSWORD-} ]]; then
    echo "The variable IOS_APPLE_ID_PASSWORD is not set."
    read -sp "IOS_APPLE_ID_PASSWORD = " IOS_APPLE_ID_PASSWORD
    echo ""
    export IOS_APPLE_ID_PASSWORD
fi

# Provisioning profiles directory
if [[ -z ${IOS_PROVISIONING_PROFILES_DIR-} ]]; then
    IOS_PROVISIONING_PROFILES_DIR="$SCRIPT_DIR/iOS Provisioning Profiles"

    echo "The variable IOS_PROVISIONING_PROFILES_DIR is not set."
    echo "Default: $IOS_PROVISIONING_PROFILES_DIR"

    export IOS_PROVISIONING_PROFILES_DIR
fi

###########################################
# Build configuration
###########################################

# The Xcode project name without file extension
# The folder with all sources is expected to hold the same name
PROJECT_NAME="MullvadVPN"

# Xcode project file
XCODE_PROJECT="$PROJECT_NAME.xcodeproj"

# Build output directory without trailing slash
BUILD_OUTPUT="$SCRIPT_DIR/Build"

# Xcode archive output
XCODE_ARCHIVE="$BUILD_OUTPUT/$PROJECT_NAME.xcarchive"

# Export options file used for producing .xcarchive
EXPORT_OPTIONS_FILE="$SCRIPT_DIR/ExportOptions.plist"

# Path to generated IPA file produced after .xcarchive export
IPA_FILE="$BUILD_OUTPUT/$PROJECT_NAME.ipa"

# Xcodebuild intermediate files directory
DERIVED_DATA_DIR="$BUILD_OUTPUT/DerivedData"

# System provisioning profiles directory
SYSTEM_PROVISIONING_PROFILES_DIR="$HOME/Library/MobileDevice/Provisioning Profiles"


###########################################
# Install provisioning profiles
###########################################

get_mobile_provisioning_uuid() {
  security cms -D -i "$1" | grep -aA1 UUID | grep -o "[-a-zA-Z0-9]\{36\}"
}

install_mobile_provisioning() {
    for filename in "$IOS_PROVISIONING_PROFILES_DIR"/*.mobileprovision; do
        [ -f "$filename" ] || continue
        local profile_uuid=$(get_mobile_provisioning_uuid "$filename")
        local target="$SYSTEM_PROVISIONING_PROFILES_DIR/$profile_uuid.mobileprovision"

        echo "Install $filename -> $target"

        cp "$filename" "$target"
    done
}

install_mobile_provisioning


###########################################
# Build Xcode project
###########################################

release_build() {
  xcodebuild \
    -project "$XCODE_PROJECT" \
    -scheme "$PROJECT_NAME" \
    -sdk iphoneos \
    -configuration Release \
    -derivedDataPath "$DERIVED_DATA_DIR" \
    $@
}

# Clean build directory
release_build clean

# Archive project
release_build archive -archivePath "$XCODE_ARCHIVE"

# Export IPA for distribution
xcodebuild \
    -exportArchive \
    -archivePath "$XCODE_ARCHIVE" \
    -exportOptionsPlist "$EXPORT_OPTIONS_FILE" \
    -exportPath "$BUILD_OUTPUT"


###########################################
# Deploy to AppStore
###########################################

if [[ "${1:-""}" == "--deploy" ]]; then
    xcrun altool \
        --upload-app --verbose \
        --type ios \
        --file "$IPA_FILE" \
        --username "$IOS_APPLE_ID" \
        --password "$IOS_APPLE_ID_PASSWORD"
else
    echo "Deployment to AppStore will not be performed."
    echo "Run with --deploy to upload the binary."
fi
