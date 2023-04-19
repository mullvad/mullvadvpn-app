#!/usr/bin/env zsh

set -euvx

# Setup the various names used in this script

# The target name for the shadowsocks-proxy dynamic library and framework
export SHADOW_SOCKS_TARGET_NAME=libshadowsocks_proxy
export SHADOW_SOCKS_FOLDER=MullvadREST/shadowsocks-proxy
export SHADOW_SOCKS_HEADER_FILE=shadowsocks.h

# The cargo target name that differentiates builds for iOS and iOS simulator
if [[ ${ARCHS_STANDARD} = "arm64 x86_64" ]]; then
  export RUST_TARGET=aarch64-apple-ios-sim
else
  export RUST_TARGET=aarch64-apple-ios
fi

# Whether the target should be built in release mode
if [[ ${CONFIGURATION} = "Release" ]]; then
  export SHOULD_BUILD_RELEASE="--release"
else
  export SHOULD_BUILD_RELEASE=""
fi

# Specify the output of the cargo build
export CARGO_TARGET_DIR="$PROJECT_DIR/build"

# Add the path to the cargo executable
export PATH="$HOME/.cargo/bin:$PATH"
if [[ -n "${DEVELOPER_SDK_DIR:-}" ]]; then
  # Add an extra library search path for build scripts and proc-macros which run on the host instead of the target
  # (macOS Big Sur does not have linkable libraries in /usr/lib/)
  export LIBRARY_PATH="${DEVELOPER_SDK_DIR}/MacOSX.sdk/usr/lib:${LIBRARY_PATH:-}"
fi

# Go to the folder that contains the Rust Shadowsocks crate
pushd ${SHADOW_SOCKS_FOLDER}

# Issue the build command along with the target and release options
cargo build --target ${RUST_TARGET} ${SHOULD_BUILD_RELEASE}

# Rewrite the load path of the dynamic library
install_name_tool -id "@rpath/Frameworks/${SHADOW_SOCKS_TARGET_NAME}.dylib" ${CARGO_TARGET_DIR}/${RUST_TARGET}/${CONFIGURATION}/${SHADOW_SOCKS_TARGET_NAME}.dylib

# Pop the folder so no special path needs to be specified for the `xcodebuild` command
popd

# Wrap the dynamic library in an .xcframework format
xcrun xcodebuild -create-xcframework -library ${CARGO_TARGET_DIR}/${RUST_TARGET}/${CONFIGURATION}/${SHADOW_SOCKS_TARGET_NAME}.dylib -headers ${SHADOW_SOCKS_FOLDER}/include/ -output output/${SHADOW_SOCKS_TARGET_NAME}.xcframework

# Copy the .xcframework file to the products directory for later consumption
cp -R output/${SHADOW_SOCKS_TARGET_NAME}.xcframework ${BUILT_PRODUCTS_DIR}

