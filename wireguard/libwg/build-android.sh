#!/usr/bin/env bash

set -eu

# Ensure we are in the correct directory for the execution of this script
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $script_dir

# Keep a GOPATH in the build directory to maintain a cache of downloaded libraries
export GOPATH=$script_dir/../../build/android-go-path/
mkdir -p $GOPATH

ARCHITECTURES="${ARCHITECTURES:-"arm arm64 x86_64 x86"}"
for arch in $ARCHITECTURES; do
    case "$arch" in
        "arm64")
            export ANDROID_C_COMPILER="${NDK_TOOLCHAIN_DIR}/aarch64-linux-android21-clang"
            export ANDROID_STRIP_TOOL="${NDK_TOOLCHAIN_DIR}/aarch64-linux-android-strip"
            export RUST_TARGET_TRIPLE="aarch64-linux-android"
            export ANDROID_ABI="arm64-v8a"
            ;;
        "x86_64")
            export ANDROID_C_COMPILER="${NDK_TOOLCHAIN_DIR}/x86_64-linux-android21-clang"
            export ANDROID_STRIP_TOOL="${NDK_TOOLCHAIN_DIR}/x86_64-linux-android-strip"
            export RUST_TARGET_TRIPLE="x86_64-linux-android"
            export ANDROID_ABI="x86_64"
            ;;
        "arm")
            export ANDROID_C_COMPILER="${NDK_TOOLCHAIN_DIR}/armv7a-linux-androideabi21-clang"
            export ANDROID_STRIP_TOOL="${NDK_TOOLCHAIN_DIR}/arm-linux-androideabi-strip"
            export RUST_TARGET_TRIPLE="armv7-linux-androideabi"
            export ANDROID_ABI="armeabi-v7a"
            ;;
        "x86")
            export ANDROID_C_COMPILER="${NDK_TOOLCHAIN_DIR}/i686-linux-android21-clang"
            export ANDROID_STRIP_TOOL="${NDK_TOOLCHAIN_DIR}/i686-linux-android-strip"
            export RUST_TARGET_TRIPLE="i686-linux-android"
            export ANDROID_ABI="x86"
            ;;
    esac

    export ANDROID_ARCH_NAME=$arch

    # Build Wireguard-Go
    echo $(pwd)
    make -f Android.mk clean

    export CFLAGS="-D__ANDROID_API__=21"

    make -f Android.mk

    # Strip and copy the libray to `android/build/extraJni/$ANDROID_ABI` to be able to build the APK
    UNSTRIPPED_LIB_PATH="../../build/lib/$RUST_TARGET_TRIPLE/libwg.so"
    STRIPPED_LIB_PATH="../../android/build/extraJni/$ANDROID_ABI/libwg.so"

    mkdir -p "$(dirname "$STRIPPED_LIB_PATH")"

    $ANDROID_STRIP_TOOL --strip-unneeded --strip-debug -o "$STRIPPED_LIB_PATH" "$UNSTRIPPED_LIB_PATH"

    rm -rf build
done

# ensure `git clean -fd` does not require root permissions
find $GOPATH -exec chmod +rw {} \;
