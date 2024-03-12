#!/usr/bin/env bash

set -euvx

if [ "$#" -gt 2 ]
then
    echo "Usage (note: only call inside xcode!):"
    echo "build-rust-library.sh <FFI_TARGET> [FFI_FEATURES]"
    exit 1
fi

# what to pass to cargo build -p, e.g. your_lib_ffi
FFI_TARGET=$1

# Enable cargo features by passing feature names to this script, i.e. build-rust-library.sh mullvad-api api-override
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

if [[ -n "${DEVELOPER_SDK_DIR:-}" ]]; then
  # Assume we're in Xcode, which means we're probably cross-compiling.
  # In this case, we need to add an extra library search path for build scripts and proc-macros,
  # which run on the host instead of the target.
  # (macOS Big Sur does not have linkable libraries in /usr/lib/.)
  export LIBRARY_PATH="${DEVELOPER_SDK_DIR}/MacOSX.sdk/usr/lib:${LIBRARY_PATH:-}"
fi

IS_SIMULATOR=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
  IS_SIMULATOR=1
fi

for arch in $ARCHS; do
  case "$arch" in
    x86_64)
      if [ $IS_SIMULATOR -eq 0 ]; then
        echo "Building for x86_64, but not a simulator build. What's going on?" >&2
        exit 2
      fi

      # Intel iOS simulator
      export CFLAGS_x86_64_apple_ios="-target x86_64-apple-ios"
      "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib $RELFLAG --target x86_64-apple-ios ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
      "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib --target x86_64-apple-ios ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
      ;;

    arm64)
      if [ $IS_SIMULATOR -eq 0 ]; then
        # Hardware iOS targets
        "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib $RELFLAG --target aarch64-apple-ios ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
        "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib --target aarch64-apple-ios ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
      else
        "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib $RELFLAG --target aarch64-apple-ios-sim ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
        "$HOME"/.cargo/bin/cargo build -p "$FFI_TARGET" --lib --target aarch64-apple-ios-sim ${FEATURE_FLAGS:+--features "$FEATURE_FLAGS"}
      fi
  esac
done
