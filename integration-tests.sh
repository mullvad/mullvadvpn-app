#!/bin/sh -e

MULLVAD_DIR="$(dirname "$0")"
PREVIOUS_DIR="$(pwd)"

if [ ! -z "$MULLVAD_DIR" ]; then
    cd "$MULLVAD_DIR"
fi

cargo build
cd mullvad-tests
cargo test --features "integration-tests"

if [ ! -z "$MULLVAD_DIR" ]; then
    cd "$PREVIOUS_DIR"
fi
