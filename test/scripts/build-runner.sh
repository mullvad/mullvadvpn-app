#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$SCRIPT_DIR/../.."
cd "$SCRIPT_DIR"

source "$REPO_DIR/scripts/utils/log"

case ${1-:""} in
    linux)
        TARGET=x86_64-unknown-linux-gnu
        shift
    ;;
    windows)
        TARGET=x86_64-pc-windows-gnu
        shift
    ;;
    macos)
        # TODO: x86
        TARGET=aarch64-apple-darwin
        shift
    ;;
    *)
        log_error "Invalid platform. Specify a valid platform as first argument"
        exit 1
esac

cargo build \
    --bin test-runner \
    --bin connection-checker \
    --release --target "${TARGET}"

# Only build runner image for Windows
if [[ $TARGET == x86_64-pc-windows-gnu ]]; then
    TARGET="$TARGET" ./build-runner-image.sh
fi
