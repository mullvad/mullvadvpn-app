#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

GRADLE_BUILD_TYPE="release"
GRADLE_TASKS=(createOssProdReleaseDistApk createPlayProdReleaseDistApk)
BUILD_BUNDLE="no"
BUNDLE_TASKS=(createPlayProdReleaseDistBundle)
RUN_PLAY_PUBLISH_TASKS="no"
PLAY_PUBLISH_TASKS=()

while [ -n "${1:-""}" ]; do
    if [[ "${1:-""}" == "--dev-build" ]]; then
        GRADLE_BUILD_TYPE="debug"
        GRADLE_TASKS=(createOssProdDebugDistApk)
        BUNDLE_TASKS=(createOssProdDebugDistBundle)
    elif [[ "${1:-""}" == "--fdroid" ]]; then
        GRADLE_BUILD_TYPE="fdroid"
        GRADLE_TASKS=(createOssProdFdroidDistApk)
        BUNDLE_TASKS=(createOssProdFdroidDistBundle)
    elif [[ "${1:-""}" == "--app-bundle" ]]; then
        BUILD_BUNDLE="yes"
    elif [[ "${1:-""}" == "--enable-play-publishing" ]]; then
        RUN_PLAY_PUBLISH_TASKS="yes"
    fi

    shift 1
done

if [[ "$GRADLE_BUILD_TYPE" == "release" ]]; then
    if [[ -n "$(git status --porcelain)" ]]; then
      echo "Dirty working directory! Will not accept that for an official release."
      exit 1
    fi

    if [ ! -f "$SCRIPT_DIR/credentials/keystore.properties" ]; then
        echo "ERROR: No keystore.properties file found" >&2
        echo "       Please configure the signing keys as described in the README" >&2
        exit 1
    fi
fi

echo "Computing build version..."
echo ""
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version versionName)
echo "Building Mullvad VPN $PRODUCT_VERSION for Android"
echo ""

if [[ "$GRADLE_BUILD_TYPE" == "release" ]]; then
    if [[ "$PRODUCT_VERSION" == *"-dev-"* ]]; then
        GRADLE_TASKS+=(createPlayDevmoleReleaseDistApk createPlayStagemoleReleaseDistApk)
        BUNDLE_TASKS+=(createPlayDevmoleReleaseDistBundle createPlayStagemoleReleaseDistBundle)
    elif [[ "$PRODUCT_VERSION" == *"-alpha"* ]]; then
        GRADLE_TASKS+=(createPlayDevmoleReleaseDistApk createPlayStagemoleReleaseDistApk)
        BUNDLE_TASKS+=(createPlayDevmoleReleaseDistBundle createPlayStagemoleReleaseDistBundle)
        PLAY_PUBLISH_TASKS=(publishPlayDevmoleReleaseBundle publishPlayStagemoleReleaseBundle publishPlayProdReleaseBundle)
    fi
fi

# Fallback to the system-wide gradle command if the gradlew script is removed.
# It is removed by the F-Droid build process before the build starts.
if [ -f "gradlew" ]; then
    GRADLE_CMD="./gradlew"
elif which gradle > /dev/null; then
    GRADLE_CMD="gradle"
else
    echo "ERROR: No gradle command found" >&2
    echo "       Please either install gradle or restore the gradlew file" >&2
    exit 2
fi

$GRADLE_CMD --console plain clean

$GRADLE_CMD --console plain "${GRADLE_TASKS[@]}"

if [[ "$BUILD_BUNDLE" == "yes" ]]; then
    $GRADLE_CMD --console plain "${BUNDLE_TASKS[@]}"
fi

# When building releases, we check that the working directory is clean before building,
# further up. Now verify that this is still true. The build process should never make the
# working directory dirty.
# This could for example happen if lockfiles are outdated, and the build process updates them.
if [[ "$GRADLE_BUILD_TYPE" == "release" ]]; then
    if [[ -n $(git status --porcelain) ]]; then
        log_error "The build made the working directory dirty!"
        log_error "This should never happen, and is explicitly forbidden for release builds!"
        exit 1
    fi
fi

if [[ "$RUN_PLAY_PUBLISH_TASKS" == "yes" && "${#PLAY_PUBLISH_TASKS[@]}" -ne 0 ]]; then
    $GRADLE_CMD --console plain "${PLAY_PUBLISH_TASKS[@]}"
fi

echo "**********************************"
echo ""
echo " The build finished successfully! "
echo " You have built:"
echo ""
echo " $PRODUCT_VERSION"
echo ""
echo " Build checksums:"
sha256sum ../dist/MullvadVPN-"$PRODUCT_VERSION"* | sed 's/^/ /'
echo ""
echo "**********************************"
