#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"


PRODUCT_VERSION="$(sed -n -e 's/^ *versionName "\([^"]*\)"$/\1/p' android/build.gradle)"
BUILD_TYPE="release"
GRADLE_BUILD_TYPE="release"
GRADLE_TASK="assembleRelease"
BUNDLE_TASK="bundleRelease"
BUILT_APK_SUFFIX="-release"
FILE_SUFFIX=""
CARGO_ARGS="--release"
EXTRA_WGGO_ARGS=""
BUILD_BUNDLE="no"

while [ ! -z "${1:-""}" ]; do
    if [[ "${1:-""}" == "--dev-build" ]]; then
        BUILD_TYPE="debug"
        GRADLE_BUILD_TYPE="debug"
        GRADLE_TASK="assembleDebug"
        BUNDLE_TASK="bundleDebug"
        BUILT_APK_SUFFIX="-debug"
        FILE_SUFFIX="-debug"
        CARGO_ARGS=""
    elif [[ "${1:-""}" == "--fdroid" ]]; then
        GRADLE_BUILD_TYPE="fdroid"
        GRADLE_TASK="assembleFdroid"
        BUNDLE_TASK="bundleFdroid"
        BUILT_APK_SUFFIX="-fdroid-unsigned"
        EXTRA_WGGO_ARGS="--no-docker"
    elif [[ "${1:-""}" == "--app-bundle" ]]; then
        BUILD_BUNDLE="yes"
    fi

    shift 1
done

if [[ "$GRADLE_BUILD_TYPE" == "release" ]]; then
    if [ ! -f "$SCRIPT_DIR/android/keystore.properties" ]; then
        echo "ERROR: No keystore.properties file found" >&2
        echo "       Please configure the signing keys as described in the README" >&2
        exit 1
    fi
fi

product_version_commit_hash=$(git rev-parse android/$PRODUCT_VERSION^{commit} || echo "")
current_head_commit_hash=$(git rev-parse HEAD^{commit})
if [[ "$BUILD_TYPE" == "debug" || $product_version_commit_hash != $current_head_commit_hash ]]; then
    PRODUCT_VERSION="${PRODUCT_VERSION}-dev-${current_head_commit_hash:0:6}"
    echo "Modifying product version to $PRODUCT_VERSION"
else
    echo "Removing old Rust build artifacts"
    cargo +stable clean
    CARGO_ARGS+=" --locked"
fi

echo "Building Mullvad VPN $PRODUCT_VERSION for Android"
pushd "$SCRIPT_DIR/android"

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
mkdir -p "build/extraJni"
popd

function restore_metadata_backups() {
    pushd "$SCRIPT_DIR"
    ./version-metadata.sh restore-backup --only-android
    mv Cargo.lock.bak Cargo.lock || true
    popd
}
trap 'restore_metadata_backups' EXIT

cp Cargo.lock Cargo.lock.bak
./version-metadata.sh inject $PRODUCT_VERSION --only-android

./wireguard/build-wireguard-go.sh --android $EXTRA_WGGO_ARGS


ARCHITECTURES="aarch64 armv7 x86_64 i686"
for ARCHITECTURE in $ARCHITECTURES; do
    case "$ARCHITECTURE" in
        "x86_64")
            TARGET="x86_64-linux-android"
            ABI="x86_64"
            ;;
        "i686")
            TARGET="i686-linux-android"
            ABI="x86"
            ;;
        "aarch64")
            TARGET="aarch64-linux-android"
            ABI="arm64-v8a"
            ;;
        "armv7")
            TARGET="armv7-linux-androideabi"
            ABI="armeabi-v7a"
            ;;
    esac

    echo "Building mullvad-daemon for $TARGET"
    cargo +stable build $CARGO_ARGS --target "$TARGET" --package mullvad-jni

    cp "$SCRIPT_DIR/target/$TARGET/$BUILD_TYPE/libmullvad_jni.so" "$SCRIPT_DIR/android/build/extraJni/$ABI/"
done

./update-relays.sh
./update-api-address.sh

cd "$SCRIPT_DIR/android"
$GRADLE_CMD --console plain "$GRADLE_TASK"

mkdir -p "$SCRIPT_DIR/dist"
cp  "$SCRIPT_DIR/android/build/outputs/apk/$GRADLE_BUILD_TYPE/android${BUILT_APK_SUFFIX}.apk" \
    "$SCRIPT_DIR/dist/MullvadVPN-${PRODUCT_VERSION}${FILE_SUFFIX}.apk"

if [[ "$BUILD_BUNDLE" == "yes" ]]; then
    $GRADLE_CMD --console plain "$BUNDLE_TASK"

    cp  "$SCRIPT_DIR/android/build/outputs/bundle/$GRADLE_BUILD_TYPE/android${BUILT_APK_SUFFIX}.aab" \
        "$SCRIPT_DIR/dist/MullvadVPN-${PRODUCT_VERSION}${FILE_SUFFIX}.aab"
fi

echo "**********************************"
echo ""
echo " The build finished successfully! "
echo " You have built:"
echo ""
echo " $PRODUCT_VERSION"
echo ""
echo "**********************************"
