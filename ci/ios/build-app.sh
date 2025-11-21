#!/usr/bin/env bash
# Buildscript to run inside a build VM to build a new IPA for the iOS app.

set -eu -o pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CHECKOUT_DIR="${SCRIPT_DIR}/mullvadvpn-app/"
CLONE_DIR="${SCRIPT_DIR}/build/"
BUILD_DIR="${CLONE_DIR}/ios"
LAST_BUILD_APP_DIR="${SCRIPT_DIR}/build-output"
LAST_BUILD_LOG="${SCRIPT_DIR}/last-build-log"


# Remove old build dir. Xcode doesn't like when just the `Build` folder is
# deleted, it is easiest to not mess up the original checkout dir and instead
# do the build in a copy. The copy can be then discarded.
rm -rf "$CLONE_DIR" || echo "Failed to remove old build directory"
cp -a "$CHECKOUT_DIR" "$CLONE_DIR"

cd "$BUILD_DIR"

# Instantiate Xcconfig templates.
for file in ./Configurations/*.template ; do cp "$file" "${file//.template/}" ; done

IOS_PROVISIONING_PROFILES_DIR="${SCRIPT_DIR}/provisioning-profiles" \
    bash build.sh | tee "$LAST_BUILD_LOG"

mkdir -p "$LAST_BUILD_APP_DIR"
cp "./Build/MullvadVPN.ipa" "$LAST_BUILD_APP_DIR"
