#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TEST_FRAMEWORK_ROOT="$SCRIPT_DIR/../.."
REPO_ROOT="$TEST_FRAMEWORK_ROOT/.."

export BUILD_RELEASE_REPOSITORY="https://releases.mullvad.net/desktop/releases"
export BUILD_DEV_REPOSITORY="https://releases.mullvad.net/desktop/builds"

function executable_not_found_in_dist_error {
    1>&2 echo "Executable \"$1\" not found in specified dist dir. Exiting."
    exit 1
}

# Returns the directory of the lib.sh script
function get_test_utls_dir {
    ## local script_path="${BASH_SOURCE[0]}"
    ## local script_dir
    ## if [[ -n "$script_path" ]]; then
    ##     script_dir="$(cd "$(dirname "$script_path")" >/dev/null && pwd)"
    ## else
    ##     script_dir="$(cd "$(dirname "$0")" >/dev/null && pwd)"
    ## fi
    ## echo "$script_dir"
    echo "$SCRIPT_DIR"
}

# Infer stable version from GitHub repo
RELEASES=$(curl -sf https://api.github.com/repos/mullvad/mullvadvpn-app/releases | jq -r '[.[] | select(((.tag_name|(startswith("android") or startswith("ios"))) | not))]')
LATEST_STABLE_RELEASE=$(jq -r '[.[] | select(.prerelease==false)] | .[0].tag_name' <<<"$RELEASES")

function get_current_version {
    local app_dir
    app_dir="$REPO_ROOT"
    if [ -n "${TEST_DIST_DIR+x}" ]; then
        if [ ! -x "${TEST_DIST_DIR%/}/mullvad-version" ]; then
            executable_not_found_in_dist_error mullvad-version
        fi
        "${TEST_DIST_DIR%/}/mullvad-version"
    else
        cargo run -q --manifest-path="$app_dir/Cargo.toml" --bin mullvad-version
    fi
}

CURRENT_VERSION=$(get_current_version)
commit=$(git rev-parse HEAD^\{commit\})
commit=${commit:0:6}

TAG=$(git describe --exact-match HEAD 2>/dev/null || echo "")

if [[ -n "$TAG" && ${CURRENT_VERSION} =~ -dev- ]]; then
    # Remove disallowed version characters from the tag
    CURRENT_VERSION+="+${TAG//[^0-9a-z_-]/}"
fi

export CURRENT_VERSION
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
