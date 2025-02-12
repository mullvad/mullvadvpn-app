#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"
TEST_DIR="$SCRIPT_DIR/../.."

if [[ "$#" -lt 1 ]]; then
    echo "usage: $0 TEST_OS" 1>&2
    exit 1
fi

TEST_OS=$1

if [[ "$(uname -s)" == "Darwin" ]]; then
    # NOTE: We only do this on macOS since we use containers otherwise
    echo "Updating Rust toolchain"
    rustup update
fi

# shellcheck source=test/scripts/test-utils.sh
source "test-utils.sh"

echo "**********************************"
echo "* Version to upgrade from: $LATEST_STABLE_RELEASE"
echo "* Version to test: $CURRENT_VERSION"
echo "**********************************"


if [[ -z "${ACCOUNT_TOKENS+x}" ]]; then
    echo "'ACCOUNT_TOKENS' must be specified" 1>&2
    exit 1
fi
if ! readarray -t tokens < "${ACCOUNT_TOKENS}"; then
    echo "Specify account numbers in 'ACCOUNT_TOKENS' file" 1>&2
    exit 1
fi
CI_LOGS_DIR="$TEST_DIR/.ci-logs"
mkdir -p "$CI_LOGS_DIR"
echo "$CURRENT_VERSION" > "$CI_LOGS_DIR/last-version.log"


echo "**********************************"
echo "* Downloading app packages"
echo "**********************************"


nice_time download_app_package "$LATEST_STABLE_RELEASE" "$TEST_OS"
nice_time download_app_package "$CURRENT_VERSION" "$TEST_OS"
nice_time download_e2e_executable "$CURRENT_VERSION" "$TEST_OS"

echo "**********************************"
echo "* Building test manager"
echo "**********************************"

cargo build -p test-manager

echo "**********************************"
echo "* Running tests"
echo "**********************************"

mkdir -p "$CI_LOGS_DIR/os/"
export TEST_REPORT="$CI_LOGS_DIR/${TEST_OS}_report"
rm -f "$TEST_REPORT"

set -o pipefail

APP_PACKAGE=$(get_app_filename "$CURRENT_VERSION" "$TEST_OS")
export APP_PACKAGE
APP_PACKAGE_TO_UPGRADE_FROM=$(get_app_filename "$LATEST_STABLE_RELEASE" "$TEST_OS")
export APP_PACKAGE_TO_UPGRADE_FROM
ACCOUNT_TOKEN=${tokens[0]} RUST_LOG=debug nice_time run_tests_for_os "${TEST_OS}"
