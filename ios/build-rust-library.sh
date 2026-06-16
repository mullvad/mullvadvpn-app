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
if [[ "$CONFIGURATION" == "Release" || "$CONFIGURATION" == "MockRelease" ]]; then
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

# Make the WireGuard data path fast even in debug builds.
#
# It is not just the crypto: the `dev` profile also turns on `debug-assertions`
# and `overflow-checks` for *every* crate, so even with `opt-level=3` the
# non-crypto hot loops (gotatun's packet/noise/device handling, the smoltcp
# stack, and our own TUN/mux glue) pay for a bounds/overflow check on every
# operation. Unoptimized ChaCha20-Poly1305 (via `ring`) caps decrypt throughput,
# and the checked glue inflates per-packet cost on top of that — together they
# can cost several x of tunnel throughput versus an optimized (e.g. wireguard-go)
# baseline, which is misleading when measuring a debug build on-device.
#
# So for the data-path crates we both raise `opt-level` and drop the debug checks
# (matching what `[profile.release(.package)]` already does for release builds).
# Scoped to this iOS build only (this script builds nothing else) via per-package
# `--config` overrides, so host/desktop builds are untouched and iteration stays
# fast (only these few crates are affected).
#
# The `mullvad-ios` FFI crate keeps `opt-level=0`: optimizing it dead-strips
# `#[no_mangle]` exports and breaks linking against the Swift side. We still drop
# its debug/overflow checks, which speeds up the per-packet glue without changing
# `opt-level` (so the exports survive).
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
)

for arch in $ARCHS; do
    case "$arch" in
        arm64)
            "$HOME"/.cargo/bin/cargo build $LOCKEDFLAG "${OPT_CONFIG[@]}" -p "$FFI_TARGET" --lib $RELFLAG --target $TARGET ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
            ;;
    esac
done
