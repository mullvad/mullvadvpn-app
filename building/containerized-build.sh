#!/usr/bin/env bash

# Helper script to build the Android app in a container.
# Uses podman unless overridden using the environment
# variable: CONTAINER_RUNNER

set -eu

REGISTRY_HOST="ghcr.io"
REGISTRY_ORG="mullvad"
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
        container_name="mullvadvpn-app-build"
        container_tag_path="$SCRIPT_DIR/linux-container-image-tag.txt"
        build_script="$REPO_MOUNT_TARGET/build.sh"
        default_build_flags="--no-docker"
        shift 1
    ;;
    android)
        container_name="mullvadvpn-app-build-android"
        container_tag_path="$SCRIPT_DIR/android-container-image-tag.txt"
        build_script="$REPO_MOUNT_TARGET/build-apk.sh"
        default_build_flags="--no-docker"
        shift 1
    ;;
    *)
        log_error "Invalid platform. Specify 'linux' or 'android' as first argument"
        exit 1
esac

full_container_name_with_tag="$REGISTRY_HOST/$REGISTRY_ORG/$container_name:$(cat "$container_tag_path")"
build_command="$build_script $default_build_flags $*"

echo ""
echo "Runner   : $CONTAINER_RUNNER"
echo "Container: $full_container_name_with_tag"
echo "Command  : $build_command"
echo ""

"$CONTAINER_RUNNER" run --rm \
    -v "$REPO_DIR":"$REPO_MOUNT_TARGET":Z \
    -v "$CARGO_TARGET_VOLUME_NAME":/root/.cargo/target:Z \
    -v "$CARGO_REGISTRY_VOLUME_NAME":/root/.cargo/registry:Z \
    -v "$GRADLE_CACHE_VOLUME_NAME":/root/.gradle:Z \
    "$full_container_name_with_tag" bash -c "$build_command"
