#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

source "$SCRIPT_DIR/scripts/test-utils.sh"

PACKAGE_FOLDER="${CACHE_FOLDER}"

if [[ -z "${ACCOUNT_TOKEN+x}" ]]; then
    echo "'ACCOUNT_TOKEN' must be specified" 1>&2
    exit 1
fi

if [[ -z "${TEST_OS+x}" ]]; then
    echo "'TEST_OS' must be specified" 1>&2
    exit 1
fi

if [[ -z "${APP_VERSION+x}" ]]; then
    echo "APP_VERSION not set, using latest stable release"
    echo "Available releases include:"
    print_available_releases
    echo "For a full list of available releases, see $BUILD_RELEASE_REPOSITORY and $BUILD_DEV_REPOSITORY"
    APP_VERSION=$LATEST_STABLE_RELEASE
fi

echo "**********************************"
echo "* Version to test: $APP_VERSION"
echo "**********************************"

echo "**********************************"
echo "* Downloading app packages"
echo "**********************************"

# Clean up old packages
find "$PACKAGE_FOLDER" -type f -mtime +5 -delete || true

nice_time download_app_package "$APP_VERSION" "$TEST_OS"
nice_time download_e2e_executable "$APP_VERSION" "$TEST_OS"


set -o pipefail
APP_PACKAGE=$(get_app_filename "$APP_VERSION" "$TEST_OS")
nice_time run_tests_for_os "${TEST_OS}"
