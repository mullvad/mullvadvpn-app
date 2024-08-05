#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# shellcheck source=test/scripts/test-utils.sh
source "scripts/test-utils.sh"

if [[ -z "${PACKAGE_DIR+x}" ]]; then
    echo "'PACKAGE_DIR' not set, defaulting to $CACHE_FOLDER for downloading app packages."
    PACKAGE_DIR="$CACHE_FOLDER"
fi

if [[ -z "${ACCOUNT_TOKEN+x}" ]]; then
    echo "'ACCOUNT_TOKEN' must be specified" 1>&2
    exit 1
fi

if [[ -z "${TEST_OS+x}" ]]; then
    echo "'TEST_OS' must be specified" 1>&2
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

# Clean up old packages
find "$PACKAGE_DIR" -type f -mtime +5 -delete || true

nice_time download_app_package "$APP_VERSION" "$TEST_OS"
nice_time download_e2e_executable "$APP_VERSION" "$TEST_OS"


set -o pipefail
APP_PACKAGE=$(get_app_filename "$APP_VERSION" "$TEST_OS")
export APP_PACKAGE
nice_time run_tests_for_os "${TEST_OS}"
