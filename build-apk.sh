#!/usr/bin/env bash

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

PRODUCT_VERSION="$(node -p "require('../gui/package.json').version" | sed -Ee 's/\.0//g')"

if [[ "${1:-""}" == "--dev-build" ]]; then
    BUILD_TYPE="debug"
    GRADLE_TASK="assembleDebug"
    APK_SUFFIX="-debug"
    CARGO_FLAGS=""
else
    BUILD_TYPE="release"
    GRADLE_TASK="assembleRelease"
    APK_SUFFIX=""
    CARGO_FLAGS="--release"

    if [ ! -f "keystore.properties" ]; then
        echo "ERROR: No keystore.properties file found" >&2
        echo "       Please configure the signing keys as described in the README" >&2
        exit 1
    fi
fi

if [[ "$BUILD_TYPE" == "debug" || "$(git describe)" != "$PRODUCT_VERSION" ]]; then
    GIT_COMMIT="$(git rev-parse --short HEAD)"
    APK_VERSION="${PRODUCT_VERSION}-dev-${GIT_COMMIT}"
else
    APK_VERSION="$PRODUCT_VERSION"
fi

ARCHITECTURES="aarch64 armv7 x86_64 i686"

./gradlew --console plain clean
mkdir -p build/extraJni

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

    . env.sh "$TARGET"
    cargo build $CARGO_FLAGS --target "$TARGET" --package mullvad-jni

    cp -a "dist-assets/binaries/$TARGET" "android/build/extraJni/$ABI"
    cp "target/$TARGET/$BUILD_TYPE/libmullvad_jni.so" "android/build/extraJni/$ABI/"
done

cd android
./gradlew --console plain "$GRADLE_TASK"

GENERATED_APK="build/outputs/apk/$BUILD_TYPE/android-$BUILD_TYPE.apk"

mkdir -p ../dist
cp "$GENERATED_APK" "../dist/MullvadVPN-${APK_VERSION}${APK_SUFFIX}.apk"
