#!/usr/bin/env bash

# Helper script to build the Android app in a container.
# Uses podman unless overridden using the environment
# variable: CONTAINER_COMMAND

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$SCRIPT_DIR"

CONTAINER_COMMAND=${CONTAINER_COMMAND:-"podman"}

image_tag=$(cat "$REPO_DIR"/building/android-container-image-tag.txt)
build_command="./build-apk.sh --no-docker $*"

printf "Building in podman using command: %s\n\n" "$build_command"
$CONTAINER_COMMAND run --rm -v "$REPO_DIR":/build ghcr.io/mullvad/mullvadvpn-app-build-android:"$image_tag" bash -c "$build_command"
