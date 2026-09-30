#!/usr/bin/env bash

set -eu

usage() {
    echo "This script downloads and tests the given app version from the build repositories."
    echo
    echo "Required environment variables:"
    echo "  - TEST_OS: Name of the VM configuration to use. List available configurations with 'cargo run --bin test-manager config vm list'"
    echo "Optional environment variables:"
    echo "  - ACCOUNT_TOKEN: Valid MullvadVPN account number (defaults to the account of TEST_ENV in the config file)"
    echo "  - APP_VERSION: The version of the app to test (defaults to the latest stable release)"
    echo "  - APP_PACKAGE_TO_UPGRADE_FROM: The package version to upgrade from (defaults to none)"
    echo "  - TEST_ENV: Name of the environment in the test-manager config to use, eg staging (defaults to mullvad.net without test locations)"
    echo "  - TEST_DIST_DIR: Relative path to a directory with prebuilt binaries as produced by scripts/build.sh."
    echo "  - TEST_FILTERS: specifies which tests to run (defaults to all)"
    echo "  - TEST_REPORT : path to save the test results in a structured format"
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# shellcheck source=test/scripts/utils/lib.sh
source "scripts/utils/lib.sh"

if [[ ("$*" == "--help") || "$*" == "-h" ]]; then
    usage
    exit 0
fi

if [[ -z "${TEST_OS+x}" ]]; then
    echo "'TEST_OS' must be specified" 1>&2
    echo
    usage
    exit 1
fi

if [[ -z "${APP_VERSION+x}" ]]; then
    echo "'APP_VERSION' not set, using latest build from the list of GitHub releases:"
    print_available_releases
    echo "For a full list of available releases you can choose from, see the stable build repository: $BUILD_RELEASE_REPOSITORY"
    echo "and the dev build repository: $BUILD_DEV_REPOSITORY"
    APP_VERSION=$LATEST_STABLE_RELEASE
fi

echo "**********************************"
echo "* Version to test: $APP_VERSION"
echo "**********************************"

echo "**********************************"
echo "* Downloading app packages"
echo "**********************************"

download_app_package "$APP_VERSION" "$TEST_OS"
download_e2e_executable "$APP_VERSION" "$TEST_OS"

if [[ -n "${APP_PACKAGE_TO_UPGRADE_FROM+x}" ]]; then
    download_app_package "$APP_PACKAGE_TO_UPGRADE_FROM" "$TEST_OS"
fi

set -o pipefail
APP_PACKAGE=$(get_app_filename "$APP_VERSION" "$TEST_OS")
export APP_PACKAGE
run_tests_for_os "${TEST_OS}"
