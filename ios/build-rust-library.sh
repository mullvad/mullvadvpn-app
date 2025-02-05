#!/usr/bin/env bash

set -euvx

if [ "$#" -gt 2 ] || [ "$#" -eq 0 ]
then
    echo "Usage (note: only call inside xcode!):"
    echo "build-rust-library.sh <FFI_TARGET> [FFI_FEATURES]"
    exit 1
fi



# what to pass to cargo build -p, e.g. your_lib_ffi
FFI_TARGET=$1

# Enable cargo features by passing feature names to this script, i.e. build-rust-library.sh mullvad-api api-override
# If more than one feature flag needs to be enabled, pass in a single argument all the features flags separated by spaces
# build-rust-library.sh mullvad-api "featureA featureB featureC"
FEATURE_FLAGS=
if [[ "$#" -eq 2 ]] ; then
FEATURE_FLAGS=$2
echo ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
fi


RELFLAG=
if [[ "$CONFIGURATION" == "Release" ]]; then
  RELFLAG=--release
fi
if [[ "$CONFIGURATION" == "MockRelease" ]]; then
  RELFLAG=--release
fi

# For whatever reason, Xcode includes its toolchain paths in the PATH variable such as
#
# /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin
# /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/appleinternal/bin
# /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/local/bin
# /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/libexec
# When this happens, cargo will be tricked into building for the wrong architecture, which will lead to linker issues down the line.
# cargo does not need to know about all this, therefore, set the path to the bare minimum
export PATH="${HOME}/.cargo/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/Library/Apple/usr/bin:"
# Since some of the dependencies come from homebrew, add it manually as well
export PATH="${PATH}:/opt/homebrew/bin:"

IS_SIMULATOR=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
  IS_SIMULATOR=1
fi

for arch in $ARCHS; do
  case "$arch" in
    arm64)
      if [ $IS_SIMULATOR -eq 0 ]; then
        # Hardware iOS targets
        rustup target add aarch64-apple-ios
      
        "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib $RELFLAG --target aarch64-apple-ios ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
        "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib --target aarch64-apple-ios ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
      else
        # iOS Simulator targets for arm64
        rustup target add aarch64-apple-ios-sim

        "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib $RELFLAG --target aarch64-apple-ios-sim ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
        "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib --target aarch64-apple-ios-sim ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
      fi
  esac
done
