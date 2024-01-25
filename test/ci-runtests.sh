#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_DIR="$SCRIPT_DIR/../"
cd "$SCRIPT_DIR"

BUILD_RELEASE_REPOSITORY="https://releases.mullvad.net/desktop/releases"
BUILD_DEV_REPOSITORY="https://releases.mullvad.net/desktop/builds"

if [[ ("$(uname -s)" == "Darwin") ]]; then
    export PACKAGES_DIR=$HOME/Library/Caches/mullvad-test/packages
elif [[ ("$(uname -s)" == "Linux") ]]; then
    export PACKAGES_DIR=$HOME/.cache/mullvad-test/packages
else
    echo "Unsupported OS" 1>&2
    exit 1
fi

if [[ "$#" -lt 1 ]]; then
    echo "usage: $0 TEST_OS" 1>&2
    exit 1
fi

TEST_OS=$1

# Infer stable version from GitHub repo
RELEASES=$(curl -sf https://api.github.com/repos/mullvad/mullvadvpn-app/releases | jq -r '[.[] | select(((.tag_name|(startswith("android") or startswith("ios"))) | not))]')
OLD_APP_VERSION=$(jq -r '[.[] | select(.prerelease==false)] | .[0].tag_name' <<<"$RELEASES")

NEW_APP_VERSION=$(cargo run -q --manifest-path="$APP_DIR/Cargo.toml" --bin mullvad-version)
commit=$(git rev-parse HEAD^\{commit\})
commit=${commit:0:6}

TAG=$(git describe --exact-match HEAD 2>/dev/null || echo "")

if [[ -n "$TAG" && ${NEW_APP_VERSION} =~ -dev- ]]; then
    NEW_APP_VERSION+="+${TAG}"
fi

echo "**********************************"
echo "* Version to upgrade from: $OLD_APP_VERSION"
echo "* Version to test: $NEW_APP_VERSION"
echo "**********************************"


if [[ -z "${ACCOUNT_TOKENS+x}" ]]; then
    echo "'ACCOUNT_TOKENS' must be specified" 1>&2
    exit 1
fi
if ! readarray -t tokens < "${ACCOUNT_TOKENS}"; then
    echo "Specify account tokens in 'ACCOUNT_TOKENS' file" 1>&2
    exit 1
fi

mkdir -p "$SCRIPT_DIR/.ci-logs"
echo "$NEW_APP_VERSION" > "$SCRIPT_DIR/.ci-logs/last-version.log"

function nice_time {
    SECONDS=0
    if "$@"; then
        result=0
    else
        result=$?
    fi
    s=$SECONDS
    echo "\"$*\" completed in $((s/60))m:$((s%60))s"
    return $result
}

# Returns 0 if $1 is a development build. `BASH_REMATCH` contains match groups
# if that is the case.
function is_dev_version {
    local pattern="(^[0-9.]+(-beta[0-9]+)?-dev-)([0-9a-z]+)(\+[0-9a-z|-]+)?$"
    if [[ "$1" =~ $pattern ]]; then
        return 0
    fi
    return 1
}

function get_app_filename {
    local version=$1
    local os=$2
    if is_dev_version "$version"; then
        # only save 6 chars of the hash
        local commit="${BASH_REMATCH[3]}"
        version="${BASH_REMATCH[1]}${commit}"
        # If the dev-version includes a tag, we need to append it to the app filename
        if [[ -n ${BASH_REMATCH[4]} ]]; then
            version="${version}${BASH_REMATCH[4]}"
        fi
    fi
    case $os in
        debian*|ubuntu*)
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

    mkdir -p "$PACKAGES_DIR"
    if [[ ! -f "$PACKAGES_DIR/$filename" ]]; then
        echo "Downloading build for $version ($os) from $url"
        curl -sf -o "$PACKAGES_DIR/$filename" "$url"
    else
        echo "Found build for $version ($os)"
    fi
}

function get_e2e_filename {
    local version=$1
    local os=$2
    if is_dev_version "$version"; then
        # only save 6 chars of the hash
        local commit="${BASH_REMATCH[3]}"
        version="${BASH_REMATCH[1]}${commit}"
    fi
    case $os in
        debian*|ubuntu*|fedora*)
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
    local version=$1
    local os=$2
    local package_repo=""

    if is_dev_version "$version"; then
        package_repo="${BUILD_DEV_REPOSITORY}"
    else
        package_repo="${BUILD_RELEASE_REPOSITORY}"
    fi

    local filename
    filename=$(get_e2e_filename "$version" "$os")
    local url="${package_repo}/$version/additional-files/$filename"

    mkdir -p "$PACKAGES_DIR"
    if [[ ! -f "$PACKAGES_DIR/$filename" ]]; then
        echo "Downloading e2e executable for $version ($os) from $url"
        curl -sf -o "$PACKAGES_DIR/$filename" "$url"
    else
        echo "Found e2e executable for $version ($os)"
    fi
}

function run_tests_for_os {
    local os=$1

    local prev_filename
    prev_filename=$(get_app_filename "$OLD_APP_VERSION" "$os")
    local cur_filename
    cur_filename=$(get_app_filename "$NEW_APP_VERSION" "$os")

    rm -f "$SCRIPT_DIR/.ci-logs/${os}_report"

    RUST_LOG=debug cargo run --bin test-manager \
        run-tests \
        --account "${ACCOUNT_TOKEN}" \
        --current-app "${cur_filename}" \
        --previous-app "${prev_filename}" \
        --test-report "$SCRIPT_DIR/.ci-logs/${os}_report" \
        "$os" 2>&1 | sed "s/${ACCOUNT_TOKEN}/\{ACCOUNT_TOKEN\}/g"
}

echo "**********************************"
echo "* Downloading app packages"
echo "**********************************"

mkdir -p "$PACKAGES_DIR"
nice_time download_app_package "$OLD_APP_VERSION" "$TEST_OS"
nice_time download_app_package "$NEW_APP_VERSION" "$TEST_OS"
nice_time download_e2e_executable "$NEW_APP_VERSION" "$TEST_OS"

echo "**********************************"
echo "* Building test runner"
echo "**********************************"

# Clean up packages. Try to keep ones that match the versions we're testing
find "$PACKAGES_DIR/" -type f ! \( -name "*${OLD_APP_VERSION}_*" -o -name "*${OLD_APP_VERSION}.*" -o -name "*${commit}*" \) -delete || true

function build_test_runner {
    local target=""
    if [[ "${TEST_OS}" =~ "debian"|"ubuntu"|"fedora" ]]; then
        target="x86_64-unknown-linux-gnu"
    elif [[ "${TEST_OS}" =~ "windows" ]]; then
        target="x86_64-pc-windows-gnu"
    elif [[ "${TEST_OS}" =~ "macos" ]]; then
        target="aarch64-apple-darwin"
    fi
    TARGET=$target ./build.sh
}

nice_time build_test_runner

echo "**********************************"
echo "* Building test manager"
echo "**********************************"

cargo build -p test-manager

echo "**********************************"
echo "* Running tests"
echo "**********************************"

mkdir -p "$SCRIPT_DIR/.ci-logs/os/"
set -o pipefail
ACCOUNT_TOKEN=${tokens[0]} nice_time run_tests_for_os "${TEST_OS}"
