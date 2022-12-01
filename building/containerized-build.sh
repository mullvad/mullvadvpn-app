#!/usr/bin/env bash

# Helper script to build the Android app in a container.
# Uses podman unless overridden using the environment
# variable: CONTAINER_RUNNER

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$SCRIPT_DIR"

CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}
REPO_MOUNT_TARGET="/build"

image_tag=$(cat "$REPO_DIR"/building/android-container-image-tag.txt)
build_command="$REPO_MOUNT_TARGET/build-apk.sh --no-docker $*"

printf "Building in $CONTAINER_RUNNER using command: %s\n\n" "$build_command"
$CONTAINER_RUNNER run --rm -v "$REPO_DIR":"$REPO_MOUNT_TARGET" ghcr.io/mullvad/mullvadvpn-app-build-android:"$image_tag" /bin/bash -c "$build_command"
