#!/usr/bin/env bash

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

REPO_DIR="$SCRIPT_DIR/../../"
TOOLCHAINS_DIR="$HOME/android-ndk-toolchains"

# Install Rust
curl -sf -L https://sh.rustup.rs > /tmp/rustup.sh
chmod +x /tmp/rustup.sh
/tmp/rustup.sh -y
source "$HOME/.cargo/env"
rustup set profile minimal
rustup target add \
    i686-linux-android \
    x86_64-linux-android \
    aarch64-linux-android \
    armv7-linux-androideabi

# Install Go
cd "$HOME"
curl -sf -L -O https://dl.google.com/go/go1.13.3.linux-amd64.tar.gz
echo "0804bf02020dceaa8a7d7275ee79f7a142f1996bfd0c39216ccb405f93f994c0 go1.13.3.linux-amd64.tar.gz" | sha256sum -c
tar -xzvf go1.13.3.linux-amd64.tar.gz
patch -p1 -f -N -r- -d "$HOME/go" < "$REPO_DIR/wireguard/libwg/goruntime-boottime-over-monotonic.diff"

# Prepare standalone NDK toolchains
mkdir "$TOOLCHAINS_DIR"
for arch in arm arm64 x86 x86_64; do
    case "$arch" in
        "arm64")
            android_lib_triple="aarch64-linux-android"
            ;;
        "x86_64")
            android_lib_triple="x86_64-linux-android"
            ;;
        "arm")
            android_lib_triple="arm-linux-androideabi"
            ;;
        "x86")
            android_lib_triple="i686-linux-android"
            ;;
    esac

    "$NDK_PATH/build/tools/make-standalone-toolchain.sh" --platform=android-21 --arch="$arch" --install-dir="$TOOLCHAINS_DIR/android21-$arch"

    for file in crtbegin_dynamic.o crtend_android.o crtbegin_so.o crtend_so.o; do
        ln -s "$TOOLCHAINS_DIR/android21-$arch/sysroot/usr/lib/$android_lib_triple/"{21/,}"$file"
    done
done

# Configure Cargo for cross-compilation
sed -e "s|{NDK_PATH}|$NDK_PATH|g" "$SCRIPT_DIR/cargo-config.toml.template" > "$HOME/.cargo/config"
