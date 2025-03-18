#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_FRAMEWORK_ROOT="$SCRIPT_DIR/../.."
REPO_ROOT="$TEST_FRAMEWORK_ROOT/.."

export BUILD_RELEASE_REPOSITORY="https://releases.mullvad.net/desktop/releases"
export BUILD_DEV_REPOSITORY="https://releases.mullvad.net/desktop/builds"

function executable_not_found_in_dist_error {
    1>&2 echo "Executable \"$1\" not found in specified dist dir. Exiting."
    exit 1
}

# Returns the directory of the lib.sh script
function get_test_utils_dir {
    echo "$SCRIPT_DIR"
}

# Infer stable version from GitHub repo
RELEASES=$(curl -sf https://api.github.com/repos/mullvad/mullvadvpn-app/releases | jq -r '[.[] | select(((.tag_name|(startswith("android") or startswith("ios"))) | not))]')
LATEST_STABLE_RELEASE=$(jq -r '[.[] | select(.prerelease==false)] | .[0].tag_name' <<<"$RELEASES")

commit=$(git rev-parse HEAD^\{commit\})
commit=${commit:0:6}

export LATEST_STABLE_RELEASE

function print_available_releases {
    for release in $(jq -r '.[].tag_name' <<<"$RELEASES"); do
        echo "$release"
    done
}

function get_package_dir {
    local package_dir
    if [[ -n "${PACKAGE_DIR+x}" ]]; then
        # Resolve the package dir to an absolute path since cargo must be invoked from the test directory
        package_dir=$(realpath "$PACKAGE_DIR")
    elif [[ ("$(uname -s)" == "Darwin") ]]; then
        package_dir="$HOME/Library/Caches/mullvad-test/packages"
    elif [[ ("$(uname -s)" == "Linux") ]]; then
        package_dir="$HOME/.cache/mullvad-test/packages"
    else
        echo "Unsupported OS" 1>&2
        exit 1
    fi

    mkdir -p "$package_dir" || exit 1
    # Clean up old packages
    find "$package_dir" -type f -mtime +5 -delete || true

    echo "$package_dir"
    return 0
}

function nice_time {
    SECONDS=0
    if "$@"; then
        result=0
    else
        result=$?
    fi
    s=$SECONDS
    echo "\"$*\" completed in $((s / 60))m:$((s % 60))s"
    return $result
}
# Matches $1 with a build version string and sets the following exported variables:
# - BUILD_VERSION: The version part of the build string (e.g., "2024.3-beta1-dev-").
# - COMMIT_HASH: The commit hash part of the build string (e.g., "abcdef").
# - TAG: The tag part of the build string (e.g., "+tag").
function parse_build_version {
    if [[ "$1" =~ (^[0-9.]+(-beta[0-9]+)?-dev-)([0-9a-z]+)(\+[0-9a-z|-]+)?$ ]]; then
        BUILD_VERSION="${BASH_REMATCH[1]}"
        COMMIT_HASH="${BASH_REMATCH[3]}"
        TAG="${BASH_REMATCH[4]}"
        return 0
    fi
    return 1
}

# Returns 0 if $1 is a development build.
function is_dev_version {
    if [[ "$1" == *"-dev-"* ]]; then
        return 0
    fi
    return 1
}

function get_e2e_filename {
    local version=$1
    local os=$2
    if is_dev_version "$version"; then
        parse_build_version "$version"
        version="${BUILD_VERSION}${COMMIT_HASH}"
    fi
    case $os in
    debian* | ubuntu* | fedora*)
        echo "app-e2e-tests-${version}-x86_64-unknown-linux-gnu"
        ;;
    windows*)
        echo "app-e2e-tests-${version}-x86_64-pc-windows-msvc.exe"
        ;;
    macos*)
        echo "app-e2e-tests-${version}-aarch64-apple-darwin"
        ;;
    *)
        echo "Unsupported target: $os" 1>&2
        return 1
        ;;
    esac
}

function get_app_filename {
    local version=$1
    local os=$2
    if is_dev_version "$version"; then
        parse_build_version "$version"
        version="${BUILD_VERSION}${COMMIT_HASH}${TAG:-}"
    fi
    case $os in
    debian* | ubuntu*)
        echo "MullvadVPN-${version}_amd64.deb"
        ;;
    fedora*)
        echo "MullvadVPN-${version}_x86_64.rpm"
        ;;
    windows*)
        echo "MullvadVPN-${version}.exe"
        ;;
    macos*)
        echo "MullvadVPN-${version}.pkg"
        ;;
    *)
        echo "Unsupported target: $os" 1>&2
        return 1
        ;;
    esac
}

function build_test_runner {
    local script_dir
    script_dir="$(get_test_utils_dir)/../"
    local test_os=${1:?Error: test os not set}
    if [[ "${test_os}" =~ "debian"|"ubuntu"|"fedora" ]]; then
        "$script_dir"/container-run.sh scripts/build/test-runner.sh linux || exit 1
    elif [[ "${test_os}" =~ "windows" ]]; then
        "$script_dir"/container-run.sh scripts/build/test-runner.sh windows || exit 1
    elif [[ "${test_os}" =~ "macos" ]]; then
        "$script_dir"/build/test-runner.sh macos || exit 1
    fi
}

function run_tests_for_os {
    local vm=$1

    if [[ -z "${ACCOUNT_TOKEN+x}" ]]; then
        echo "'ACCOUNT_TOKEN' must be specified" 1>&2
        exit 1
    fi

    if [ -n "${TEST_DIST_DIR+x}" ]; then
        if [ ! -x "${TEST_DIST_DIR%/}/test-runner" ]; then
            executable_not_found_in_dist_error test-runner
        fi

        echo "**********************************"
        echo "* Using test-runner in $TEST_DIST_DIR"
        echo "**********************************"
    else
        echo "**********************************"
        echo "* Building test runner"
        echo "**********************************"
        nice_time build_test_runner "$vm"
    fi

    echo "**********************************"
    echo "* Running tests"
    echo "**********************************"

    local upgrade_package_arg
    if [[ -z "${APP_PACKAGE_TO_UPGRADE_FROM+x}" ]]; then
        echo "'APP_PACKAGE_TO_UPGRADE_FROM' env not set, not testing upgrades"
        upgrade_package_arg=()
    else
        upgrade_package_arg=(--app-package-to-upgrade-from "${APP_PACKAGE_TO_UPGRADE_FROM}")
    fi

    if [[ -z "${TEST_REPORT+x}" ]]; then
        echo "'TEST_REPORT' env not set, not saving test report"
        test_report_arg=()
    else
        test_report_arg=(--test-report "${TEST_REPORT}")
    fi

    local package_dir
    package_dir=$(get_package_dir)
    local test_dir
    test_dir=$(get_test_utils_dir)/../..
    read -ra test_filters_arg <<<"${TEST_FILTERS:-}" # Split the string by words into an array
    pushd "$test_dir"
    if [ -n "${TEST_DIST_DIR+x}" ]; then
        if [ ! -x "${TEST_DIST_DIR%/}/test-manager" ]; then
            executable_not_found_in_dist_error test-manager
        fi
        test_manager="${TEST_DIST_DIR%/}/test-manager"
        runner_dir_flag=("--runner-dir" "$TEST_DIST_DIR")
    else
        # Build & run test-manager
        test_manager="cargo run --bin test-manager"
        runner_dir_flag=()
    fi

    if [ -n "${MULLVAD_HOST+x}" ]; then
        mullvad_host_arg=("--mullvad-host" "$MULLVAD_HOST")
    else
        mullvad_host_arg=()
    fi

    if ! RUST_LOG_STYLE=always $test_manager run-tests \
        --account "${ACCOUNT_TOKEN:?Error: ACCOUNT_TOKEN not set}" \
        --app-package "${APP_PACKAGE:?Error: APP_PACKAGE not set}" \
        "${upgrade_package_arg[@]}" \
        "${test_report_arg[@]}" \
        --package-dir "${package_dir}" \
        --vm "$vm" \
        --openvpn-certificate "${OPENVPN_CERTIFICATE:-"assets/openvpn.ca.crt"}" \
        "${mullvad_host_arg[@]}" \
        "${test_filters_arg[@]}" \
        "${runner_dir_flag[@]}" \
        2>&1 | sed -r "s/${ACCOUNT_TOKEN}/\{ACCOUNT_TOKEN\}/g"; then
        echo "Test run failed"
        exit 1
    fi
    popd
}

# Build the current version of the app and move the package to the package folder
# Currently unused, but may be useful in the future
function build_current_version {
    local app_dir
    app_dir="$REPO_ROOT"
    local current_version
    pushd "$app_dir"
    current_version=$(cargo run --package mullvad-version)
    popd
    local app_filename
    # TODO: TEST_OS must be set to local OS manually, should be set automatically
    app_filename=$(get_app_filename "$current_version" "${TEST_OS:?Error: TEST_OS not set}")
    local package_dir
    package_dir=$(get_package_dir)
    local app_package="$package_dir"/"$app_filename"

    local gui_test_filename
    gui_test_filename=$(get_e2e_filename "$current_version" "$TEST_OS")
    local gui_test_bin="$package_dir"/"$gui_test_filename"

    if [ ! -f "$app_package" ]; then
        pushd "$app_dir"
        if [[ $(git diff --quiet) ]]; then
            echo "WARNING: the app repository contains uncommitted changes, this script will only rebuild the app package when the git hash changes"
        fi
        ./build.sh
        popd
        echo "Moving '$(realpath "$app_dir/dist/$app_filename")' to '$(realpath "$app_package")'"
        mv -n "$app_dir"/dist/"$app_filename" "$app_package"
    else
        echo "App package for current version already exists at $app_package, skipping build"
    fi

    if [ ! -f "$gui_test_bin" ]; then
        pushd "$app_dir"/gui
        npm run build-test-executable
        popd
        echo "Moving '$(realpath "$app_dir/dist/$gui_test_filename")' to '$(realpath "$gui_test_bin")'"
        mv -n "$app_dir"/dist/"$gui_test_filename" "$gui_test_bin"
    else
        echo "GUI e2e executable for current version already exists at $gui_test_bin, skipping build"
    fi
}

function download_app_package {
    local version=$1
    local os=$2
    local package_repo=""

    if is_dev_version "$version"; then
        package_repo="${BUILD_DEV_REPOSITORY}"
    else
        package_repo="${BUILD_RELEASE_REPOSITORY}"
    fi

    local filename
    filename=$(get_app_filename "$version" "$os")
    local url="${package_repo}/$version/$filename"

    local package_dir
    package_dir=$(get_package_dir)
    if [[ ! -f "$package_dir/$filename" ]]; then
        echo "Downloading build for $version ($os) from $url"
        if ! curl -sf -o "$package_dir/$filename" "$url"; then
            echo "Failed to download package from $url (hint: build may not exist, check the url)" 1>&2
            exit 1
        fi
    else
        echo "App package for version $version ($os) already exists at $package_dir/$filename, skipping download"
    fi
}

function download_e2e_executable {
    local version=${1:?Error: version not set}
    local os=${2:?Error: os not set}
    local package_repo

    if is_dev_version "$version"; then
        package_repo="${BUILD_DEV_REPOSITORY}"
    else
        package_repo="${BUILD_RELEASE_REPOSITORY}"
    fi

    local filename
    filename=$(get_e2e_filename "$version" "$os")
    local url="${package_repo}/$version/additional-files/$filename"

    local package_dir
    package_dir=$(get_package_dir)
    if [[ ! -f "$package_dir/$filename" ]]; then
        echo "Downloading e2e executable for $version ($os) from $url"
        if ! curl -sf -o "$package_dir/$filename" "$url"; then
            echo "Failed to download package from $url (hint: build may not exist, check the url)" 1>&2
            exit 1
        fi
    else
        echo "GUI e2e executable for version $version ($os) already exists at $package_dir/$filename, skipping download"
    fi
}
