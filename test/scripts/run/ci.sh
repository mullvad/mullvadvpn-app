#!/usr/bin/env bash

# TODO: Break this down into multiple, smaller scripts and compose them in this file.

set -eu

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_FRAMEWORK_ROOT="$SCRIPT_DIR/../.."
TEST_DIR="$TEST_FRAMEWORK_ROOT"

cd "$SCRIPT_DIR"

# Parse arguments
# TODO: Add support for either passing in --account-tokens or reading from env variable.
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

# shellcheck source=test/scripts/utils/lib.sh
source "../utils/lib.sh"
# shellcheck source=test/scripts/utils/download.sh
source "../utils/download.sh" # TODO: Do not source it, call it instead.

echo "**********************************"
echo "* Version to upgrade from: $LATEST_STABLE_RELEASE"
echo "* Version to test: $CURRENT_VERSION"
echo "**********************************"

# TODO: Add support for either passing in --account-tokens or reading from env variable.
if [[ -z "${ACCOUNT_TOKENS+x}" ]]; then
    echo "'ACCOUNT_TOKENS' must be specified" 1>&2
    exit 1
fi
if ! readarray -t tokens <"${ACCOUNT_TOKENS}"; then
    echo "Specify account numbers in 'ACCOUNT_TOKENS' file" 1>&2
    exit 1
fi

# TODO: Can we get rid of this? Seemse excessive / leaves a trail
CI_LOGS_DIR="$TEST_DIR/.ci-logs"
mkdir -p "$CI_LOGS_DIR"
echo "$CURRENT_VERSION" >"$CI_LOGS_DIR/last-version.log"

# TODO: This should def be it's own step in the GitHub actions workflow

echo "**********************************"
echo "* Downloading app packages"
echo "**********************************"

nice_time download_app_package "$LATEST_STABLE_RELEASE" "$TEST_OS"
nice_time download_app_package "$CURRENT_VERSION" "$TEST_OS"
nice_time download_e2e_executable "$CURRENT_VERSION" "$TEST_OS"

# TODO: This should def be it's own step in the GitHub actions workflow

echo "**********************************"
echo "* Running tests"
echo "**********************************"

# TODO: Should we really care about logging in this script?

mkdir -p "$CI_LOGS_DIR/os/"
export TEST_REPORT="$CI_LOGS_DIR/${TEST_OS}_report"
rm -f "$TEST_REPORT"

set -o pipefail

APP_PACKAGE=$(get_app_filename "$CURRENT_VERSION" "$TEST_OS")
export APP_PACKAGE
APP_PACKAGE_TO_UPGRADE_FROM=$(get_app_filename "$LATEST_STABLE_RELEASE" "$TEST_OS")
export APP_PACKAGE_TO_UPGRADE_FROM
ACCOUNT_TOKEN=${tokens[0]} RUST_LOG=debug nice_time run_tests_for_os "${TEST_OS}"
