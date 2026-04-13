#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

GRADLE_BUILD_TYPE="release"
GRADLE_TASKS=(createOssProdReleaseDistApk createPlayProdReleaseDistApk)
BUILD_BUNDLE="no"
BUNDLE_TASKS=(createPlayProdReleaseDistBundle)
SKIP_CLEAN_CHECK="no"
OSS_ONLY="no"

while [ -n "${1:-""}" ]; do
    if [[ "${1:-""}" == "--dev-build" ]]; then
        GRADLE_BUILD_TYPE="debug"
        GRADLE_TASKS=(createOssProdDebugDistApk)
        BUNDLE_TASKS=(createOssProdDebugDistBundle)
    elif [[ "${1:-""}" == "--oss-only" ]]; then
        OSS_ONLY="yes"
        GRADLE_TASKS=(createOssProdReleaseDistApk)
        BUNDLE_TASKS=(createOssProdReleaseDistBundle)
    elif [[ "${1:-""}" == "--skip-clean-check" ]]; then
        SKIP_CLEAN_CHECK="yes"
    elif [[ "${1:-""}" == "--app-bundle" ]]; then
        BUILD_BUNDLE="yes"
    fi

    shift 1
done

function assert_clean_working_directory {
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "Dirty working directory! Will not accept that for an official release."
        exit 1
    fi
}

if [[ "$GRADLE_BUILD_TYPE" == "release" && "$SKIP_CLEAN_CHECK" == "no" ]]; then
    assert_clean_working_directory
fi

echo "Computing build version..."
echo ""
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version versionName)
echo "Building Mullvad VPN $PRODUCT_VERSION for Android"
echo ""

if [[ "$GRADLE_BUILD_TYPE" == "release" && "$OSS_ONLY" == "no" ]]; then
    if [[ "$PRODUCT_VERSION" == *"-alpha"* || "$PRODUCT_VERSION" == *"-dev-"* ]]; then
        GRADLE_TASKS+=(
            createPlayDevmoleReleaseDistApk
            createPlayStagemoleReleaseDistApk
        )
        BUNDLE_TASKS+=(
            createPlayDevmoleReleaseDistBundle
            createPlayStagemoleReleaseDistBundle
        )
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
if [[ "$GRADLE_BUILD_TYPE" == "release" && "$SKIP_CLEAN_CHECK" == "no" ]]; then
    assert_clean_working_directory
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
