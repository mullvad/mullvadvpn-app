#!/usr/bin/env bash
set -e

MULLVAD_DIR="$(cd "$(dirname "$0")"; pwd -P)"

if [ ! -z "$MULLVAD_DIR" ]; then
    pushd "$MULLVAD_DIR"
fi

cargo build
cd mullvad-tests
cargo test --features "integration-tests"

if [ ! -z "$MULLVAD_DIR" ]; then
    popd "$PREVIOUS_DIR"
fi
