#!/usr/bin/env bash

set -e

for arch in arm arm64 x86_64 x86; do
    case "$arch" in
        "arm64")
            export ANDROID_LLVM_TRIPLE="aarch64-linux-android"
            export ANDROID_LIB_TRIPLE="aarch64-linux-android"
            export RUST_TARGET_TRIPLE="aarch64-linux-android"
            export RUST_LLVM_ARCH="aarch64"
            export ANDROID_ABI="arm64-v8a"
            ;;
        "x86_64")
            export ANDROID_LLVM_TRIPLE="x86_64-linux-android"
            export ANDROID_LIB_TRIPLE="x86_64-linux-android"
            export RUST_TARGET_TRIPLE="x86_64-linux-android"
            export RUST_LLVM_ARCH="x86_64"
            export ANDROID_ABI="x86_64"
            ;;
        "arm")
            export ANDROID_LLVM_TRIPLE="armv7a-linux-androideabi"
            export ANDROID_LIB_TRIPLE="arm-linux-androideabi"
            export RUST_TARGET_TRIPLE="armv7-linux-androideabi"
            export RUST_LLVM_ARCH="armv7"
            export ANDROID_ABI="armeabi-v7a"
            ;;
        "x86")
            export ANDROID_LLVM_TRIPLE="i686-linux-android"
            export ANDROID_LIB_TRIPLE="i686-linux-android"
            export RUST_TARGET_TRIPLE="i686-linux-android"
            export RUST_LLVM_ARCH="i686"
            export ANDROID_ABI="i686"
            ;;
    esac

    export ANDROID_ARCH_NAME="$arch"
    export ANDROID_TOOLCHAIN_ROOT="/opt/android/toolchains/android21-${RUST_LLVM_ARCH}"
    export ANDROID_SYSROOT="${ANDROID_TOOLCHAIN_ROOT}/sysroot"
    export ANDROID_C_COMPILER="${ANDROID_TOOLCHAIN_ROOT}/bin/${ANDROID_LLVM_TRIPLE}21-clang"

    # Build OpenSSL
    export PATH="$PATH:${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin"

    # Build Wireguard-Go
    echo $(pwd)
    make -f Android.mk clean

    export CFLAGS="-D__ANDROID_API__=21"
    export LDFLAGS="-L${ANDROID_SYSROOT}/usr/lib/${ANDROID_LIB_TRIPLE}/21"

    make -f Android.mk
    # Copy build artifacts to `build/libs/$RUST_TARGET_TRIPLE` to be able to build `mullvad-jni`
    chmod 777 ../android/build/
    chmod 777 ../android/build/extraJni
    chmod 777 ../android/build/extraJni/*
    mkdir -p ../build/lib/$RUST_TARGET_TRIPLE
    cp ../android/build/extraJni/$ANDROID_ABI/libwg.so ../build/lib/$RUST_TARGET_TRIPLE
done
