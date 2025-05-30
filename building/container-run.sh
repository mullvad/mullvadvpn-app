#!/usr/bin/env bash

# Gives you a shell or runs a given command in the Android or Linux build container,
# as designated by the *-container-image.txt files. Uses podman unless overridden using the
# environment variable `CONTAINER_RUNNER`. Note that this script uses named
# docker volumes that can be overridden using environment variables (see the
# beginning of the script).
#
# Usage: $ container-run.sh <linux/android> [command ...]

set -eu

REPO_MOUNT_TARGET="/build"
CARGO_TARGET_VOLUME_NAME=${CARGO_TARGET_VOLUME_NAME:-"cargo-target"}
CARGO_REGISTRY_VOLUME_NAME=${CARGO_REGISTRY_VOLUME_NAME:-"cargo-registry"}
GRADLE_CACHE_VOLUME_NAME=${GRADLE_CACHE_VOLUME_NAME:-"gradle-cache"}
ANDROID_CREDENTIALS_DIR=${ANDROID_CREDENTIALS_DIR:-""}
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}
# Temporarily do not use mold for linking by default due to it causing build errors.
# There's a separate issue (DES-1177) to address this problem.
# Build servers also opt out of this and instead use GNU ld.
USE_MOLD=${USE_MOLD:-"false"}

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$SCRIPT_DIR"

source "$REPO_DIR/scripts/utils/log"

case ${1-:""} in
    linux)
        container_image_name=$(cat "$SCRIPT_DIR/linux-container-image.txt")
        shift 1
    ;;
    android)
        container_image_name=$(cat "$SCRIPT_DIR/android-container-image.txt")
        optional_gradle_cache_volume=(-v "$GRADLE_CACHE_VOLUME_NAME:/root/.gradle:Z")

        if [ -n "$ANDROID_CREDENTIALS_DIR" ]; then
            optional_android_credentials_volume=(-v "$ANDROID_CREDENTIALS_DIR:$REPO_MOUNT_TARGET/android/credentials:Z")
        fi

        shift 1
    ;;
    *)
        log_error "Invalid platform. Specify 'linux' or 'android' as first argument"
        exit 1
esac

optional_mold=""
if [[ "$USE_MOLD" == "true" ]]; then
    optional_mold="mold -run"
fi

set -x
exec "$CONTAINER_RUNNER" run --rm -it \
    -v "/$REPO_DIR:$REPO_MOUNT_TARGET:Z" \
    -v "$CARGO_TARGET_VOLUME_NAME:/cargo-target:Z" \
    -v "$CARGO_REGISTRY_VOLUME_NAME:/root/.cargo/registry:Z" \
    "${optional_gradle_cache_volume[@]}" \
    "${optional_android_credentials_volume[@]}" \
    "$container_image_name" bash -c "$optional_mold $*"
