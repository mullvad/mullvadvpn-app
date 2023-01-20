#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "Computing build version..."
echo ""
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version versionName)
echo "Building Mullvad VPN $PRODUCT_VERSION for Android"
echo ""

BUILD_TYPE="release"
GRADLE_BUILD_TYPE="release"
GRADLE_TASK="assembleRelease"
BUNDLE_TASK="bundleRelease"
BUILT_APK_SUFFIX="-release"
FILE_SUFFIX=""
CARGO_ARGS="--release"
EXTRA_WGGO_ARGS=""
BUILD_BUNDLE="no"
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"target"}
SKIP_STRIPPING=${SKIP_STRIPPING:-"no"}

while [ ! -z "${1:-""}" ]; do
    if [[ "${1:-""}" == "--dev-build" ]]; then
        BUILD_TYPE="debug"
        GRADLE_BUILD_TYPE="debug"
        GRADLE_TASK="assembleDebug"
        BUNDLE_TASK="bundleDebug"
        BUILT_APK_SUFFIX="-debug"
        FILE_SUFFIX="-debug"
        CARGO_ARGS="--features api-override"
    elif [[ "${1:-""}" == "--fdroid" ]]; then
        GRADLE_BUILD_TYPE="fdroid"
        GRADLE_TASK="assembleFdroid"
        BUNDLE_TASK="bundleFdroid"
        BUILT_APK_SUFFIX="-fdroid-unsigned"
        EXTRA_WGGO_ARGS="--no-docker"
    elif [[ "${1:-""}" == "--app-bundle" ]]; then
        BUILD_BUNDLE="yes"
    elif [[ "${1:-""}" == "--no-docker" ]]; then
        EXTRA_WGGO_ARGS="--no-docker"
    elif [[ "${1:-""}" == "--skip-stripping" ]]; then
        SKIP_STRIPPING="yes"
    fi

    shift 1
done

if [[ "$GRADLE_BUILD_TYPE" == "release" ]]; then
    if [ ! -f "$SCRIPT_DIR/android/credentials/keystore.properties" ]; then
        echo "ERROR: No keystore.properties file found" >&2
        echo "       Please configure the signing keys as described in the README" >&2
        exit 1
    fi
fi

if [[ "$BUILD_TYPE" == "release" && "$PRODUCT_VERSION" != *"-dev-"* ]]; then
    echo "Removing old Rust build artifacts"
    cargo clean
    CARGO_ARGS+=" --locked"
fi

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
mkdir -p "app/build/extraJni"
popd

./wireguard/build-wireguard-go.sh --android $EXTRA_WGGO_ARGS

for ARCHITECTURE in ${ARCHITECTURES:-aarch64 armv7 x86_64 i686}; do
    case "$ARCHITECTURE" in
        "x86_64")
            LLVM_TRIPLE="x86_64-linux-android"
            TARGET="x86_64-linux-android"
            ABI="x86_64"
            ;;
        "i686")
            LLVM_TRIPLE="i686-linux-android"
            TARGET="i686-linux-android"
            ABI="x86"
            ;;
        "aarch64")
            LLVM_TRIPLE="aarch64-linux-android"
            TARGET="aarch64-linux-android"
            ABI="arm64-v8a"
            ;;
        "armv7")
            LLVM_TRIPLE="arm-linux-androideabi"
            TARGET="armv7-linux-androideabi"
            ABI="armeabi-v7a"
            ;;
    esac

    echo "Building mullvad-daemon for $TARGET"
    cargo build $CARGO_ARGS --target "$TARGET" --package mullvad-jni

    STRIP_TOOL="${NDK_TOOLCHAIN_DIR}/${LLVM_TRIPLE}-strip"
    TARGET_LIB_PATH="$SCRIPT_DIR/android/app/build/extraJni/$ABI/libmullvad_jni.so"
    UNSTRIPPED_LIB_PATH="$CARGO_TARGET_DIR/$TARGET/$BUILD_TYPE/libmullvad_jni.so"

    if [[ "$SKIP_STRIPPING" == "yes" ]]; then
        cp "$UNSTRIPPED_LIB_PATH" "$TARGET_LIB_PATH"
    else
        $STRIP_TOOL --strip-debug --strip-unneeded -o "$TARGET_LIB_PATH" "$UNSTRIPPED_LIB_PATH"
    fi
done

echo "Updating relays.json..."
cargo run --bin relay_list $CARGO_ARGS > build/relays.json

cd "$SCRIPT_DIR/android"
$GRADLE_CMD --console plain "$GRADLE_TASK"

mkdir -p "$SCRIPT_DIR/dist"
cp  "$SCRIPT_DIR/android/app/build/outputs/apk/$GRADLE_BUILD_TYPE/app${BUILT_APK_SUFFIX}.apk" \
    "$SCRIPT_DIR/dist/MullvadVPN-${PRODUCT_VERSION}${FILE_SUFFIX}.apk"

if [[ "$BUILD_BUNDLE" == "yes" ]]; then
    $GRADLE_CMD --console plain "$BUNDLE_TASK"

    cp  "$SCRIPT_DIR/android/app/build/outputs/bundle/$GRADLE_BUILD_TYPE/app${BUILT_APK_SUFFIX}.aab" \
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
