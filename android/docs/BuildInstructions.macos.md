# Build Instructions for macOS

This document will guide you to setup your development environment on macOS. It has been
tested on a clean install of macOS Ventura 13.5.1 on a M2 MacBook.

> __*WARNING:*__ This guide will not apply the [wireguard-go patch](https://git.zx2c4.com/wireguard-android/tree/tunnel/tools/libwg-go/goruntime-boottime-over-monotonic.diff)
> as done in Linux build instructions which may affect app performance.

## 1. Install Prerequisites

> __*NOTE:*__ Following instructions assume that you have [brew](https://brew.sh/) installed.

If you do not have Android Studio installed, install it:
```bash
brew install --cask android-studio
```

Install the following packages:
```bash
brew install protobuf gcc go openjdk@17 rustup-init
```

> __*NOTE:*__ Ensure that you setup `openjdk@17` to be the active JDK, follow instructions in
> installation of openjdk@17 or use a tool like [jEnv](https://www.jenv.be/).

Finish the install of `rustup`:
```bash
rustup-init
```

## 2. Install SDK Tools and Android NDK Toolchain
Open Android Studio -> Tools -> SDK Manager, and install `Android SDK Command-line Tools (latest)`.

Install the necessary Android SDK tools
```bash
~/Library/Android/sdk/cmdline-tools/latest/bin/sdkmanager "platforms;android-33" "build-tools;30.0.3" "platform-tools" "ndk;25.2.9519653"
```

Install Android targets
```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

Export the following environmental variables, and possibly store them for example in your
`~/.zprofile` or `~/.zshrc` file:
```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/25.2.9519653"
export NDK_TOOLCHAIN_DIR="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export AR_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_i686_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export CC_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/aarch64-linux-android26-clang"
export CC_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi26-clang"
export CC_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/x86_64-linux-android26-clang"
export CC_i686_linux_android="$NDK_TOOLCHAIN_DIR/i686-linux-android26-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/aarch64-linux-android26-clang"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi26-clang"
export CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/i686-linux-android26-clang"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/x86_64-linux-android26-clang"
```

## 4. Debug build
Run the build script in the root of the project to assemble all the native libraries and the app:

```bash
./build-apk.sh --dev-build --no-docker
```

Once the build is complete you should receive a message looking similar to this:
```
**********************************

 The build finished successfully!
 You have built:

 2023.5-dev-9ac934

**********************************
```

Your native binaries have now been built, any subsequent builds that does not have changes to the
native code can be done in Android Studio or using gradle.

# Build options and configuration

For configuring signing or options to your build continue with the general [build instructions](../BuildInstructions.md).
