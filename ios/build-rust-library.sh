#!/usr/bin/env bash

set -euvx

if [ "$#" -gt 2 ] || [ "$#" -eq 0 ]
then
    echo "Usage (note: only call inside xcode!):"
    echo "build-rust-library.sh <CARGO_PACKAGE> [\"CARGO_FEATURES\"]"
    exit 1
fi

# what to pass to cargo build -p, e.g. your_lib_ffi
CARGO_PACKAGE=$1

CARGO_ARGS=""

# Enable cargo features by passing feature names to this script, i.e. build-rust-library.sh mullvad-api api-override
# If more than one feature flag needs to be enabled, pass in a single argument all the features flags separated by spaces
# build-rust-library.sh mullvad-api "featureA featureB featureC"
if [[ "$#" -eq 2 ]] ; then
    CARGO_ARGS+="--features $2"
    echo "Building with these features: $2"
fi

RELFLAG=
if [[ "$CONFIGURATION" == "Release" || "$CONFIGURATION" == "MockRelease" ]]; then
    RELFLAG=--release

    # Release builds are not allowed to have outdated lockfiles.
    CARGO_ARGS+=" --locked"
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

TARGET=aarch64-apple-ios
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
  TARGET=aarch64-apple-ios-sim
fi

for arch in $ARCHS; do
    case "$arch" in
        arm64)
            "$HOME"/.cargo/bin/cargo build -p "$CARGO_PACKAGE" --lib $RELFLAG --target $TARGET $CARGO_ARGS
            "$HOME"/.cargo/bin/cargo build -p "$CARGO_PACKAGE" --lib --target $TARGET $CARGO_ARGS
            ;;
    esac
done
