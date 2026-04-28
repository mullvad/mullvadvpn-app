#!/usr/bin/env bash

# Builds the Android or Linux app in the current build container.
# See the `container-run.sh` script for possible configuration.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$SCRIPT_DIR"

source "$REPO_DIR/scripts/utils/log"

platform=${1-:""}
case $platform in
    linux)
        build_command=("./build.sh")
        shift 1
    ;;
    android)
        shift 1
        # Map old release arg to the new task for compatibility.
        # Can be removed once the build server has deployed the
        # run.sh script from this commit.
        [ "${1:-}" = "--app-bundle" ] && { shift; set -- fullRelease "$@"; }
        # Default to the debug task if no task provided.
        [ $# -eq 0 ] && set -- debug
        build_command=("./android/gradlew" "-p" "android" "--console" "plain")
    ;;
    *)
        log_error "Invalid platform. Specify 'linux' or 'android' as first argument"
        exit 1
esac

set -x
exec "$SCRIPT_DIR/container-run.sh" "$platform" "${build_command[@]}" "$@"
