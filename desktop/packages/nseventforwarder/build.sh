#!/usr/bin/env bash

set -eu

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --target)
            TARGET_TRIPLE="$2"
            shift
            shift
            ;;
        *)
            echo "Unknown parameter: $1"
            exit 1
            ;;
    esac
done

# Fail if TARGET_TRIPLE is not set
if [[ -z ${TARGET_TRIPLE-} ]]; then
    echo "The variable TARGET_TRIPLE is not set."
    echo "Set it using the --target flag"
    echo "Available targets: aarch64-apple-darwin, x86_64-apple-darwin"
    exit 1
fi

# Map the target to an output folder (where the dylib / node binary will end up).
# This is neon-convention.
case "$TARGET_TRIPLE" in
    aarch64-apple-darwin) PLATFORM_DIR_NAME=darwin-arm64;;
    x86_64-apple-darwin) PLATFORM_DIR_NAME=darwin-x64;;
    *)
        echo "Unknown target: $TARGET_TRIPLE"
        echo "Available targets: aarch64-apple-darwin, x86_64-apple-darwin"
        exit 1
        ;;
esac

if [[ "$(uname -s)" == "Darwin" ]]; then
    # We rely (heavily) on a pre-defined CARGO_TARGET_DIR, so don't let any user override it
    unset CARGO_TARGET_DIR
    npm run cargo-build -- --release --target "$TARGET_TRIPLE"
    # Copy the neon library to the correct dists folder, which is what node will
    # pick up when loading the library at runtime.
    PLATFORM_DIR="dist/$PLATFORM_DIR_NAME"
    mkdir -p $PLATFORM_DIR
    cp "target/$TARGET_TRIPLE/release/libnseventforwarder.dylib" "$PLATFORM_DIR/index.node"
fi
