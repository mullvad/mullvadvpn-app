#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$SCRIPT_DIR/.."
cd "$SCRIPT_DIR"

source "$REPO_DIR/scripts/utils/log"

if [[ -z ${TARGET:-""} ]]; then
    log_error "TARGET must be specified"
    exit 1
fi

source "$REPO_DIR/scripts/utils/log"

cargo build \
    --bin test-runner \
    --bin connection-checker \
    --release --target "${TARGET}"

# Only build runner image for Windows
if [[ $TARGET == x86_64-pc-windows-gnu ]]; then
    ./scripts/build-runner-image.sh
fi
