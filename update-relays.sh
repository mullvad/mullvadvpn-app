#!/usr/bin/env bash

echo "Updating relay list..."
set -e

if [[ "$1" == "--release" ]]; then
    CARGO_TOOLCHAIN="+stable"
    CARGO_ARGS="--release"
else
    CARGO_TOOLCHAIN=""
    CARGO_ARGS=""
fi

cargo $CARGO_TOOLCHAIN run --bin relay_list "$CARGO_ARGS" > dist-assets/relays.json
