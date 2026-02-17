#!/usr/bin/env bash

# This script builds an unsigned .xcarchive that can be signed later
# using resign-archive.sh. This is useful when distribution signing keys
# are not available on the build machine.
set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

###########################################
# Build configuration
###########################################

PROJECT_NAME="MullvadVPN"
XCODE_PROJECT_DIR="$SCRIPT_DIR/$PROJECT_NAME.xcodeproj"
BUILD_OUTPUT_DIR="$SCRIPT_DIR/Build"
XCODE_ARCHIVE_DIR="$BUILD_OUTPUT_DIR/$PROJECT_NAME.xcarchive"
DERIVED_DATA_DIR="$BUILD_OUTPUT_DIR/DerivedData"

###########################################
# Build unsigned archive
###########################################

release_build() {
  xcodebuild \
    -project "$XCODE_PROJECT_DIR" \
    -scheme "$PROJECT_NAME" \
    -sdk iphoneos \
    -configuration Release \
    -derivedDataPath "$DERIVED_DATA_DIR" \
    -disableAutomaticPackageResolution \
    COMPILER_INDEX_STORE_ENABLE=NO \
    CODE_SIGNING_ALLOWED=NO \
    CODE_SIGNING_REQUIRED=NO \
    CODE_SIGN_IDENTITY="" \
    "$@"
}

release_build clean
release_build archive -archivePath "$XCODE_ARCHIVE_DIR"

echo ""
echo "Unsigned archive created at: $XCODE_ARCHIVE_DIR"
echo "Use resign-archive.sh to sign and export an IPA."
