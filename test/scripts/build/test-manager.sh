#!/usr/bin/env bash

set -eu

# Build `test-manager`
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TEST_FRAMEWORK_ROOT="$SCRIPT_DIR/../.."
REPO_DIR="$TEST_FRAMEWORK_ROOT/.."

# shellcheck disable=SC1091
source "$REPO_DIR/scripts/utils/log"

build_linux() {
    cd "$TEST_FRAMEWORK_ROOT"
    # Build the test manager
    cargo build -p test-manager --release
}

case ${1-:""} in
    linux)
        build_linux
        shift
    ;;
    *)
        log_error "Invalid platform. Specify a valid platform as first argument"
        exit 1
esac
