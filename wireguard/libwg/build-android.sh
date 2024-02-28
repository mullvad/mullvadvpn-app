#!/usr/bin/env bash

set -eu

# Ensure we are in the correct directory for the execution of this script
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$script_dir"

# Keep a GOPATH in the build directory to maintain a cache of downloaded libraries
export GOPATH=$script_dir/../../build/android-go-path/
mkdir -p "$GOPATH"

ANDROID_STRIP_TOOL="${NDK_TOOLCHAIN_DIR}/llvm-strip"

for arch in ${ARCHITECTURES:-armv7 aarch64 x86_64 i686}; do
    case "$arch" in
        "aarch64")
            export ANDROID_C_COMPILER="${NDK_TOOLCHAIN_DIR}/aarch64-linux-android26-clang"
            export RUST_TARGET_TRIPLE="aarch64-linux-android"
            export ANDROID_ABI="arm64-v8a"
            export ANDROID_ARCH_NAME="arm64"
            ;;
        "x86_64")
            export ANDROID_C_COMPILER="${NDK_TOOLCHAIN_DIR}/x86_64-linux-android26-clang"
            export RUST_TARGET_TRIPLE="x86_64-linux-android"
            export ANDROID_ABI="x86_64"
            export ANDROID_ARCH_NAME="x86_64"
            ;;
        "armv7")
            export ANDROID_C_COMPILER="${NDK_TOOLCHAIN_DIR}/armv7a-linux-androideabi26-clang"
            export RUST_TARGET_TRIPLE="armv7-linux-androideabi"
            export ANDROID_ABI="armeabi-v7a"
            export ANDROID_ARCH_NAME="arm"
            ;;
        "i686")
            export ANDROID_C_COMPILER="${NDK_TOOLCHAIN_DIR}/i686-linux-android26-clang"
            export RUST_TARGET_TRIPLE="i686-linux-android"
            export ANDROID_ABI="x86"
            export ANDROID_ARCH_NAME="x86"
            ;;
    esac

    # Build Wireguard-Go
    pwd
    make -f Android.mk clean
    make -f Android.mk

    # Strip and copy the library to `android/build/extraJni/$ANDROID_ABI` to be able to build the APK
    UNSTRIPPED_LIB_PATH="../../build/lib/$RUST_TARGET_TRIPLE/libwg.so"
    STRIPPED_LIB_PATH="../../android/app/build/extraJni/$ANDROID_ABI/libwg.so"

    # Create the directories with RWX permissions for all users so that the build server can clean
    # the directories afterwards
    mkdir -p "$(dirname "$STRIPPED_LIB_PATH")"
    chmod 777 "$(dirname "$STRIPPED_LIB_PATH")"

    $ANDROID_STRIP_TOOL --strip-unneeded --strip-debug -o "$STRIPPED_LIB_PATH" "$UNSTRIPPED_LIB_PATH"

    # Set permissions so that the build server can clean the outputs afterwards
    chmod 777 "$STRIPPED_LIB_PATH"

    rm -rf build
done

# ensure `git clean -fd` does not require root permissions
find "$GOPATH" -exec chmod +rw {} \;
