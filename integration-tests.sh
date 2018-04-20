#!/usr/bin/env bash

MULLVAD_DIR="$(cd "$(dirname "$0")"; pwd -P)"

pushd "$MULLVAD_DIR"

cd mullvad-daemon \
    && cargo build --features "testing" \
    && cd ../mullvad-tests \
    && cargo test --features "integration-tests"

RESULT="$?"
popd
exit "$RESULT"
