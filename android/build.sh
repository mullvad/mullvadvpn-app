#!/usr/bin/env bash

set -e

cd "$(dirname "$0")"

product_version="$(node -p "require('../gui/package.json').version" | sed -Ee 's/\.0//g')"

if [ "$1" = "--dev-build" ]; then
    build_type="debug"
    gradle_task="assembleDebug"
    apk_suffix="-debug"
    cargo_flags=""
else
    build_type="release"
    gradle_task="assembleRelease"
    apk_suffix=""
    cargo_flags"--release"

    if [ ! -f "keystore.properties" ]; then
        echo "ERROR: No keystore.properties file found" >&2
        echo "       Please configure the signing keys as described in the README" >&2
        exit 1
    fi
fi

if [ "$build_type" = "debug" ] || [ "$(git describe)" != "$product_version" ]; then
    git_commit="$(git rev-parse --short HEAD)"
    apk_version="${product_version}-dev-${git_commit}"
else
    apk_version="$product_version"
fi

architectures="aarch64 armv7 x86_64 i686"

./gradlew --console plain clean
mkdir -p build/extraJni

cd ..

for architecture in $architectures; do
    case "$architecture" in
        "x86_64")
            target="x86_64-linux-android"
            abi="x86_64"
            ;;
        "i686")
            target="i686-linux-android"
            abi="x86"
            ;;
        "aarch64")
            target="aarch64-linux-android"
            abi="arm64-v8a"
            ;;
        "armv7")
            target="armv7-linux-androideabi"
            abi="armeabi-v7a"
            ;;
    esac

    . env.sh "$target"
    cargo build $cargo_flags --target "$target" --package mullvad-jni

    cp -a "dist-assets/binaries/$target" "android/build/extraJni/$abi"
    cp "target/$target/$build_type/libmullvad_jni.so" "android/build/extraJni/$abi/"
done

cd android
./gradlew --console plain "$gradle_task"

generated_apk="build/outputs/apk/$build_type/android-$build_type.apk"

mkdir -p ../dist
cp "$generated_apk" "../dist/MullvadVPN-${apk_version}${apk_suffix}.apk"
