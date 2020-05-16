# Sourcing this file prepares the environment for building inside the F-Droid build server

# Ensure Cargo tools are accessible
source "$HOME/.cargo/env"

# Ensure Go compiler is accessible
export GOROOT="$HOME/go"
export PATH="$PATH:$GOROOT/bin"

# Ensure Rust crates know which tools to use for cross-compilation
export TOOLCHAINS_DIR="$HOME/android-ndk-toolchains"

export AR_i686_linux_android="$TOOLCHAINS_DIR/android21-x86/bin/i686-linux-android-ar"
export AR_x86_64_linux_android="$TOOLCHAINS_DIR/android21-x86_64/bin/x86_64-linux-android-ar"
export AR_aarch64_linux_android="$TOOLCHAINS_DIR/android21-arm64/bin/aarch64-linux-android-ar"
export AR_armv7_linux_androideabi="$TOOLCHAINS_DIR/android21-arm/bin/arm-linux-androideabi-ar"

export CC_i686_linux_android="$TOOLCHAINS_DIR/android21-x86/bin/i686-linux-android21-clang"
export CC_x86_64_linux_android="$TOOLCHAINS_DIR/android21-x86_64/bin/x86_64-linux-android21-clang"
export CC_aarch64_linux_android="$TOOLCHAINS_DIR/android21-arm64/bin/aarch64-linux-android21-clang"
export CC_armv7_linux_androideabi="$TOOLCHAINS_DIR/android21-arm/bin/armv7a-linux-androideabi21-clang"

# Ensure the C cross-compilers are accessible to the libwg-go build
export ANDROID_TOOLCHAIN_ROOT_arm="$TOOLCHAINS_DIR/android21-arm"
export ANDROID_TOOLCHAIN_ROOT_x86="$TOOLCHAINS_DIR/android21-x86"
export ANDROID_TOOLCHAIN_ROOT_arm64="$TOOLCHAINS_DIR/android21-arm64"
export ANDROID_TOOLCHAIN_ROOT_x86_64="$TOOLCHAINS_DIR/android21-x86_64"
