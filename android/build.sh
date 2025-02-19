#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "Computing build version..."
echo ""
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version versionName)
echo "Building Mullvad VPN $PRODUCT_VERSION for Android"
echo ""

GRADLE_BUILD_TYPE="release"
GRADLE_TASKS=(createOssProdReleaseDistApk createPlayProdReleaseDistApk)
BUILD_BUNDLE="no"
BUNDLE_TASKS=(createPlayProdReleaseDistBundle)
RUN_PLAY_PUBLISH_TASKS="no"
PLAY_PUBLISH_TASKS=()
LOCAL_PROPERTIES_FILE="local.properties"

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
    if [ ! -f "$SCRIPT_DIR/credentials/keystore.properties" ]; then
        echo "ERROR: No keystore.properties file found" >&2
        echo "       Please configure the signing keys as described in the README" >&2
        exit 1
    fi
fi

if [[ "$GRADLE_BUILD_TYPE" == "release" ]]; then
    if [[ "$PRODUCT_VERSION" == *"-dev-"* ]]; then
        GRADLE_TASKS+=(createPlayDevmoleReleaseDistApk createPlayStagemoleReleaseDistApk)
        BUNDLE_TASKS+=(createPlayDevmoleReleaseDistBundle createPlayStagemoleReleaseDistBundle)
    elif [[ "$PRODUCT_VERSION" == *"-alpha"* ]]; then
        echo "Removing old Rust build artifacts"
        GRADLE_TASKS+=(createPlayStagemoleReleaseDistApk)
        BUNDLE_TASKS+=(createPlayStagemoleReleaseDistBundle)
        PLAY_PUBLISH_TASKS=(publishPlayStagemoleReleaseBundle)
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
md5sum ../dist/MullvadVPN-"$PRODUCT_VERSION"* | sed 's/^/ /'
echo ""
if [[ -f "$LOCAL_PROPERTIES_FILE" ]] && grep -q "^CARGO_TARGETS=" "$LOCAL_PROPERTIES_FILE"; then
    echo " CARGO_TARGETS is set in $LOCAL_PROPERTIES_FILE, build may not be reproducible!"
fi
echo ""
echo "**********************************"
