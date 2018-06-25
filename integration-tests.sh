#!/usr/bin/env bash

MULLVAD_DIR="$(cd "$(dirname "$0")"; pwd -P)"

pushd "$MULLVAD_DIR"

cargo build \
    && cd mullvad-tests \
    && cargo test --features "integration-tests" -- --test-threads=1

RESULT="$?"
popd
exit "$RESULT"
