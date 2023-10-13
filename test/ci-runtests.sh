#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_DIR="$SCRIPT_DIR/../"
cd "$SCRIPT_DIR"

MAX_CONCURRENT_JOBS=1

BUILD_RELEASE_REPOSITORY="https://releases.mullvad.net/releases"
BUILD_DEV_REPOSITORY="https://releases.mullvad.net/builds/desktop"

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

if [[ -z "${TEST_OSES+x}" ]]; then
    echo "'TEST_OSES' must be specified" 1>&2
    exit 1
fi

# Try to parse $TEST_OSES from the environment
# https://www.shellcheck.net/wiki/SC2206
IFS=" " read -r -a TEST_OSES <<< "${TEST_OSES:-}"

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
    if $@; then
        result=0
    else
        result=$?
    fi
    s=$SECONDS
    echo "\"$@\" completed in $(($s/60))m:$(($s%60))s"
    return $result
}

function account_token_from_index {
    local index
    index=$(( $1 % ${#tokens[@]} ))
    echo ${tokens[$index]}
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
    if is_dev_version $version; then
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

    if is_dev_version $version; then
        package_repo="${BUILD_DEV_REPOSITORY}"
    else
        package_repo="${BUILD_RELEASE_REPOSITORY}"
    fi

    local filename=$(get_app_filename $version $os)
    local url="${package_repo}/$version/$filename"

    # TODO: integrity check

    echo "Downloading build for $version ($os) from $url"
    mkdir -p "$SCRIPT_DIR/packages/"
    if [[ ! -f "$SCRIPT_DIR/packages/$filename" ]]; then
        curl -sf -o "$SCRIPT_DIR/packages/$filename" $url
    fi
}

function get_e2e_filename {
    local version=$1
    local os=$2
    if is_dev_version $version; then
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

    if is_dev_version $version; then
        package_repo="${BUILD_DEV_REPOSITORY}"
    else
        package_repo="${BUILD_RELEASE_REPOSITORY}"
    fi

    local filename=$(get_e2e_filename $version $os)
    local url="${package_repo}/$version/additional-files/$filename"

    echo "Downloading e2e executable for $version ($os) from $url"
    mkdir -p "$SCRIPT_DIR/packages/"
    if [[ ! -f "$SCRIPT_DIR/packages/$filename" ]]; then
        curl -sf -o "$SCRIPT_DIR/packages/$filename" $url
    fi
}

function run_tests_for_os {
    local os=$1

    local prev_filename=$(get_app_filename $OLD_APP_VERSION $os)
    local cur_filename=$(get_app_filename $NEW_APP_VERSION $os)

    rm -f "$SCRIPT_DIR/.ci-logs/${os}_report"

    RUST_LOG=debug cargo run --bin test-manager \
        run-tests \
        --account "${ACCOUNT_TOKEN}" \
        --current-app "${cur_filename}" \
        --previous-app "${prev_filename}" \
        --test-report "$SCRIPT_DIR/.ci-logs/${os}_report" \
        "$os" 2>&1 | sed "s/${ACCOUNT_TOKEN}/\{ACCOUNT_TOKEN\}/g"
    return ${PIPESTATUS[0]}
}

echo "**********************************"
echo "* Building test runners"
echo "**********************************"

# Clean up packages. Try to keep ones that match the versions we're testing
find "${SCRIPT_DIR}/packages/" -type f ! \( -name "*${OLD_APP_VERSION}_*" -o -name "*${OLD_APP_VERSION}.*" -o -name "*${NEW_APP_VERSION}*" \) -delete || true

function build_test_runners {
    for os in "${TEST_OSES[@]}"; do
        nice_time download_app_package $OLD_APP_VERSION $os || true
        nice_time download_app_package $NEW_APP_VERSION $os || true
        nice_time download_e2e_executable $NEW_APP_VERSION $os || true
    done

    local targets=()
    if [[ "${TEST_OSES[*]}" =~ "debian"|"ubuntu"|"fedora" ]]; then
        targets+=("x86_64-unknown-linux-gnu")
    fi
    if [[ "${TEST_OSES[*]}" =~ "windows" ]]; then
        targets+=("x86_64-pc-windows-gnu")
    fi
    if [[ "${TEST_OSES[*]}" =~ "macos" ]]; then
        targets+=("aarch64-apple-darwin")
    fi

    for target in "${targets[@]}"; do
        TARGET=$target ./build.sh
    done
}

nice_time build_test_runners

echo "**********************************"
echo "* Building test manager"
echo "**********************************"

cargo build -p test-manager

#
# Launch tests in all VMs
#

echo "**********************************"
echo "* Running tests"
echo "**********************************"

i=0
testjobs=""

for os in "${TEST_OSES[@]}"; do

    if [[ $i -gt 0 ]]; then
        # Certain things are racey during setup, like obtaining a pty.
        sleep 5
    fi

    mkdir -p "$SCRIPT_DIR/.ci-logs/os/"

    token=$(account_token_from_index $i)

    ACCOUNT_TOKEN=$token nice_time run_tests_for_os "$os" &> "$SCRIPT_DIR/.ci-logs/os/${os}.log" &
    testjobs[i]=$!

    ((i=i+1))

    # Limit number of concurrent jobs to $MAX_CONCURRENT_JOBS
    while :; do
        count=0
        for ((j=0; j<$i; j++)); do
            if ps -p "${testjobs[$j]}" &> /dev/null; then
                ((count=count+1))
            fi
        done
        if [[ $count -lt $MAX_CONCURRENT_JOBS ]]; then
            break
        fi
        sleep 10
    done

done

#
# Wait for them to finish
#

i=0
failed_builds=0

for os in "${TEST_OSES[@]}"; do
    if wait -fn ${testjobs[$i]}; then
        echo "**********************************"
        echo "* TESTS SUCCEEDED FOR OS: $os"
        echo "**********************************"
        tail -n 1 "$SCRIPT_DIR/.ci-logs/os/${os}.log"
    else
        let "failed_builds=failed_builds+1"

        echo "**********************************"
        echo "* TESTS FAILED FOR OS: $os"
        echo "* BEGIN LOGS"
        echo "**********************************"
        echo ""

        cat "$SCRIPT_DIR/.ci-logs/os/${os}.log"

        echo ""
        echo "**********************************"
        echo "* END LOGS FOR OS: $os"
        echo "**********************************"
    fi

    echo ""
    echo ""

    ((i=i+1))
done

#
# Generate table of test results
#
touch "$SCRIPT_DIR/.ci-logs/results.html"

report_paths=()
for os in "${TEST_OSES[@]}"; do
    report_paths=("${report_paths[@]}" "$SCRIPT_DIR/.ci-logs/${os}_report")
done

cargo run --bin test-manager \
    format-test-reports "${report_paths[@]}" \
    > "$SCRIPT_DIR/.ci-logs/results.html" || true

exit $failed_builds
