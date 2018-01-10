#!/usr/bin/env bash

set -e

export MACOSX_DEPLOYMENT_TARGET="10.7"

cargo +stable build --release

# Strip debugging symbols from the binaries. This saves a lot of space.
strip ./target/release/mullvad-daemon
strip ./target/release/mullvad
strip ./target/release/problem-report
