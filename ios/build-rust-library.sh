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
LOCKEDFLAG=
LIBFOLDER=debug
if [[ "$CONFIGURATION" == "Release" || "$CONFIGURATION" == "MockRelease" ]]; then
    LIBFOLDER=release
    RELFLAG=--release
    LOCKEDFLAG=--locked
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

# To make GotaTun fast enough to not be bothersome in DEBUG builds, lets
# disable debug assertions, overflow checks and set the optimization level to 3
# for the relevant crates. Relevant being on the data path for traffic between
# the tunnel device and the UDP socket.
OPT_CONFIG=(
    --config 'profile.dev.package.gotatun.opt-level=3'
    --config 'profile.dev.package.gotatun.debug-assertions=false'
    --config 'profile.dev.package.gotatun.overflow-checks=false'
    --config 'profile.dev.package.smoltcp.opt-level=3'
    --config 'profile.dev.package.smoltcp.debug-assertions=false'
    --config 'profile.dev.package.smoltcp.overflow-checks=false'
    --config 'profile.dev.package.ring.opt-level=3'
    --config 'profile.dev.package.chacha20poly1305.opt-level=3'
    --config 'profile.dev.package.chacha20poly1305.debug-assertions=false'
    --config 'profile.dev.package.chacha20poly1305.overflow-checks=false'
    --config 'profile.dev.package.chacha20.opt-level=3'
    --config 'profile.dev.package.chacha20.overflow-checks=false'
    --config 'profile.dev.package.poly1305.opt-level=3'
    --config 'profile.dev.package.poly1305.overflow-checks=false'
    --config 'profile.dev.package.mullvad-ios.debug-assertions=false'
    --config 'profile.dev.package.mullvad-ios.overflow-checks=false'
    --config 'profile.dev.package.mullvad-ios.opt-level=2'
)

for arch in $ARCHS; do
    case "$arch" in
        arm64)
            MODIFIED_FILE="$CONFIGURATION_BUILD_DIR/modified_$LIBFOLDER"
            LIB=
            if [[ -z  "${CARGO_TARGET_DIR}" ]]; then
                LIB="../target/$TARGET/$LIBFOLDER/libmullvad_ios.a"
            else
                LIB="$CARGO_TARGET_DIR/$TARGET/$LIBFOLDER/libmullvad_ios.a"
            fi

            OUT_DIR="MullvadRustRuntime/generated"
            OUT_DIR_TMP="$CONFIGURATION_BUILD_DIR/generated"

            echo "Building libmullvad_ios for $TARGET..."
            time "$HOME"/.cargo/bin/cargo build $LOCKEDFLAG "${OPT_CONFIG[@]}" -p "$FFI_TARGET" --lib $RELFLAG --target $TARGET ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}

            MODIFIED_DATE=$(date -r "$LIB")
            CACHED="no-checksum"
            if [ -e "$MODIFIED_FILE" ]; then
                CACHED=$(cat "$MODIFIED_FILE")
            fi
            if [ "$MODIFIED_DATE" = "$CACHED" ]; then
                echo "No notable file changes; remove '$MODIFIED_FILE' to force update"
                break
            fi

            echo "Generating Swift bindings from $LIB..."
            time xcrun --sdk macosx "$HOME"/.cargo/bin/cargo run -p mullvad-ios --features uniffi-cli --bin uniffi-bindgen -- \
                generate \
                --library "$LIB" \
                --language swift \
                --config ../mullvad-ios/uniffi.toml \
                --out-dir "$OUT_DIR_TMP"

            # uniffi names the FFI header after `ffi_module_name`; rename it and place it in the
            # include dir alongside the cbindgen header so the framework module exposes it.
            if ! cmp -s "$OUT_DIR_TMP/MullvadRustRuntimeProxy.h" MullvadRustRuntime/include/mullvad_uniffi.h; then
                mv "$OUT_DIR_TMP/MullvadRustRuntimeProxy.h" MullvadRustRuntime/include/mullvad_uniffi.h
            fi


            # uniffi only emits a swiftlint directive; also exempt the generated file from
            # swift-format (ios/format.sh lint), matching the Maybenot.swift convention.
            sed -i '' '1s;^;// swift-format-ignore-file\n;' "$OUT_DIR_TMP/mullvad_uniffi.swift"

            if ! cmp -s "$OUT_DIR_TMP/mullvad_uniffi.swift" "$OUT_DIR/mullvad_uniffi.swift"; then
                mv "$OUT_DIR_TMP/mullvad_uniffi.swift" "$OUT_DIR/mullvad_uniffi.swift"
                echo "Done. Generated:"
                echo "  $OUT_DIR/mullvad_uniffi.swift"
                echo "  MullvadRustRuntime/include/mullvad_uniffi.h"

                echo "$MODIFIED_DATE" > $MODIFIED_FILE
            fi
            ;;
    esac
done
