#!/usr/bin/env bash

# Builds the Android or Linux app in the current build container, as designated
# by the *-container-image.txt files. Uses podman unless overridden using the
# environment variable `CONTAINER_RUNNER`. Note that this script uses named
# docker volumes that can be overridden using enviornment variables (see the
# beginning of the script).

set -eu

REPO_MOUNT_TARGET="/build"
CARGO_TARGET_VOLUME_NAME=${CARGO_TARGET_VOLUME_NAME:-"cargo-target"}
CARGO_REGISTRY_VOLUME_NAME=${CARGO_REGISTRY_VOLUME_NAME:-"cargo-registry"}
GRADLE_CACHE_VOLUME_NAME=${GRADLE_CACHE_VOLUME_NAME:-"gradle-cache"}
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$SCRIPT_DIR"

source "$REPO_DIR/scripts/utils/log"

case ${1-:""} in
    linux)
        container_image_name=$(cat "$SCRIPT_DIR/linux-container-image.txt")
        build_command=("$REPO_MOUNT_TARGET/build.sh")
        shift 1
    ;;
    android)
        container_image_name=$(cat "$SCRIPT_DIR/android-container-image.txt")
        build_command=("$REPO_MOUNT_TARGET/build-apk.sh" "--no-docker")
        optional_gradle_cache_volume=(-v "$GRADLE_CACHE_VOLUME_NAME:/root/.gradle:Z")
        shift 1
    ;;
    *)
        log_error "Invalid platform. Specify 'linux' or 'android' as first argument"
        exit 1
esac

set -x
exec "$CONTAINER_RUNNER" run --rm -it \
    -v "$REPO_DIR:$REPO_MOUNT_TARGET:Z" \
    -v "$CARGO_TARGET_VOLUME_NAME:/root/.cargo/target:Z" \
    -v "$CARGO_REGISTRY_VOLUME_NAME:/root/.cargo/registry:Z" \
    "${optional_gradle_cache_volume[@]}" \
    "$container_image_name" \
    "${build_command[@]}" "$@"
