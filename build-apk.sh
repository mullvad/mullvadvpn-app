#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

bash build-wireguard-go.sh --android

PRODUCT_VERSION="$(node -p "require('$SCRIPT_DIR/gui/package.json').version" | sed -Ee 's/\.0//g')"

if [[ "${1:-""}" == "--dev-build" ]]; then
    BUILD_TYPE="debug"
    GRADLE_TASK="assembleDebug"
    APK_SUFFIX="-debug"
    CARGO_ARGS=""
else
    BUILD_TYPE="release"
    GRADLE_TASK="assembleRelease"
    APK_SUFFIX=""
    CARGO_ARGS="--release"

    if [ ! -f "$SCRIPT_DIR/android/keystore.properties" ]; then
        echo "ERROR: No keystore.properties file found" >&2
        echo "       Please configure the signing keys as described in the README" >&2
        exit 1
    fi
fi

if [[ "$BUILD_TYPE" == "debug" || "$(git describe)" != "$PRODUCT_VERSION" ]]; then
    GIT_COMMIT="$(git rev-parse HEAD | head -c 6)"
    PRODUCT_VERSION="${PRODUCT_VERSION}-dev-${GIT_COMMIT}"
    echo "Modifying product version to $PRODUCT_VERSION"
else
    echo "Removing old Rust build artifacts"
    cargo +stable clean
    CARGO_ARGS+=" --locked"
fi

pushd "$SCRIPT_DIR/android"
./gradlew --console plain clean
mkdir -p "build/extraJni"
popd

function restore_metadata_backups() {
    pushd "$SCRIPT_DIR"
    ./version_metadata.sh restore-backup
    mv Cargo.lock.bak Cargo.lock || true
    popd
}
trap 'restore_metadata_backups' EXIT

cp Cargo.lock Cargo.lock.bak
./version_metadata.sh inject $PRODUCT_VERSION

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
    source env.sh "$TARGET"
    cargo +stable build $CARGO_ARGS --target "$TARGET" --package mullvad-jni

    cp -a "$SCRIPT_DIR/dist-assets/binaries/$TARGET" "$SCRIPT_DIR/android/build/extraJni/$ABI"
    cp "$SCRIPT_DIR/target/$TARGET/$BUILD_TYPE/libmullvad_jni.so" "$SCRIPT_DIR/android/build/extraJni/$ABI/"
done

cd "$SCRIPT_DIR/android"
./gradlew --console plain "$GRADLE_TASK"

mkdir -p "$SCRIPT_DIR/dist"
cp  "$SCRIPT_DIR/android/build/outputs/apk/$BUILD_TYPE/android-$BUILD_TYPE.apk" \
    "$SCRIPT_DIR/dist/MullvadVPN-${PRODUCT_VERSION}${APK_SUFFIX}.apk"

echo "**********************************"
echo ""
echo " The build finished successfully! "
echo " You have built:"
echo ""
echo " $PRODUCT_VERSION"
echo ""
echo "**********************************"
