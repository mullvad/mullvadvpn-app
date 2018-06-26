#!/usr/bin/env bash

if [ "$UID" -ne 0 ]; then
    echo "WARNING: Not running as root, some tests may fail" >&2
fi

MULLVAD_DIR="$(cd "$(dirname "$0")"; pwd -P)"

pushd "$MULLVAD_DIR"

cargo build \
    && cd mullvad-tests \
    && cargo test --features "integration-tests" -- --test-threads=1

RESULT="$?"
popd
exit "$RESULT"
