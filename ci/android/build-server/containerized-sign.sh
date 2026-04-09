#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}
YUBIKEY_PATH=$(readlink -f /dev/android-jks-signing-key)
CONTAINER_IMAGE_NAME=$(cat "$SCRIPT_DIR/../../../building/android-container-image.txt")

ARTIFACT_DIR=${1:?'Usage: sign.sh <artifact-dir>'}
if [[ ! -d "$ARTIFACT_DIR" ]]; then
    echo "Error: not a directory: $ARTIFACT_DIR"
    exit 1
fi

if [[ -z ${YUBIKEY_PIN-} ]]; then
    echo "YUBIKEY_PIN pin must be set."
    exit 1
fi

printf '%s' "$YUBIKEY_PIN" | "$CONTAINER_RUNNER" secret create --replace YUBIKEY_PIN -
cleanup() { "$CONTAINER_RUNNER" secret rm YUBIKEY_PIN 2>/dev/null || true; }
trap cleanup EXIT

"$CONTAINER_RUNNER" run --rm -it -q \
    --device "$YUBIKEY_PATH:/dev/yubikey" \
    --secret YUBIKEY_PIN,type=env \
    -v "$SCRIPT_DIR/wait-for-pcscd.sh:/wait-for-pcscd.sh:Z" \
    -v "$SCRIPT_DIR/sign.sh:/sign.sh:Z" \
    -v "$ARTIFACT_DIR:/artifact_dir:Z" \
    -w "/artifact_dir" \
    --entrypoint /wait-for-pcscd.sh \
    "$CONTAINER_IMAGE_NAME" \
    bash -c '/sign.sh *'
