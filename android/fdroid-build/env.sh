#!/usr/bin/env bash
# Sourcing this file prepares the environment for building inside the F-Droid build server

# Ensure Cargo tools are accessible
source "$HOME/.cargo/env"

# Ensure Go compiler is accessible
export GOROOT="$HOME/go"
export PATH="$PATH:$GOROOT/bin"

# Ensure Rust crates know which tools to use for cross-compilation
export NDK_TOOLCHAIN_DIR="$NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/bin"

export AR_i686_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/llvm-ar"

export CC_i686_linux_android="$NDK_TOOLCHAIN_DIR/i686-linux-android26-clang"
export CC_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/x86_64-linux-android26-clang"
export CC_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/aarch64-linux-android26-clang"
export CC_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi26-clang"
