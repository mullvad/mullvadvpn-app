#!/usr/bin/env bash

# Builds the Android or Linux app in the current build container, as designated
# by the *-container-image.txt files. Uses podman unless overridden using the
# environment variable `CONTAINER_RUNNER`. Note that this script uses named
# docker volumes for caching between builds (see script for more details).

set -eu

REPO_MOUNT_TARGET="/build"
CARGO_TARGET_VOLUME_NAME="cargo-target"
CARGO_REGISTRY_VOLUME_NAME="cargo-registry"
GRADLE_CACHE_VOLUME_NAME="gradle-cache"
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$SCRIPT_DIR"

source "$REPO_DIR/scripts/utils/log"

case ${1-:""} in
    linux)
        container_image_name=$(cat "$SCRIPT_DIR/linux-container-image.txt")
        build_script="$REPO_MOUNT_TARGET/build.sh"
        build_script_args="$*"
        shift 1
    ;;
    android)
        container_image_name=$(cat "$SCRIPT_DIR/android-container-image.txt")
        build_script="$REPO_MOUNT_TARGET/build-apk.sh"
        build_script_args="--no-docker $*"
        shift 1
    ;;
    *)
        log_error "Invalid platform. Specify 'linux' or 'android' as first argument"
        exit 1
esac

log_info ""
log_info "Runner   : $CONTAINER_RUNNER"
log_info "Container: $container_image_name"
log_info "Command  : $build_script $build_script_args"
log_info ""

"$CONTAINER_RUNNER" run --rm \
    -v "$REPO_DIR":"$REPO_MOUNT_TARGET":Z \
    -v "$CARGO_TARGET_VOLUME_NAME":/root/.cargo/target:Z \
    -v "$CARGO_REGISTRY_VOLUME_NAME":/root/.cargo/registry:Z \
    -v "$GRADLE_CACHE_VOLUME_NAME":/root/.gradle:Z \
    "$container_image_name" "$build_script" "${build_script_args[@]}"
