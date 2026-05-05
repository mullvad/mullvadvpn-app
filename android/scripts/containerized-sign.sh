#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}
CONTAINER_IMAGE_NAME=$(cat "$SCRIPT_DIR/../../building/android-container-image.txt")

SIGN_DIR=${1:?'Usage: containerized-sign.sh <sign-dir> <bash-command>'}

if [[ ! -d "$SIGN_DIR" ]]; then
    echo "Error: not a directory: $SIGN_DIR"
    exit 1
fi

if [[ -z "$2" ]]; then
    echo "Missing command to execute, use containerized-sign.sh <sign-dir> <bash-command>"
    exit 1
fi

if [[ -z ${YUBIKEY_PIN-} ]]; then
    echo "YUBIKEY_PIN pin must be set."
    exit 1
fi

if [[ -z ${YUBIKEY_PATH-} ]]; then
    echo "YUBIKEY_PATH must be set."
    exit 1
fi

if [[ -n ${OVERRIDE_PROVIDER_CONFIG-} ]]; then
    optional_override_provider_config=(-v "$OVERRIDE_PROVIDER_CONFIG:/usr/local/etc/pkcs11_java.cfg:Z")
fi

printf '%s' "$YUBIKEY_PIN" | "$CONTAINER_RUNNER" secret create --replace YUBIKEY_PIN -
cleanup() { "$CONTAINER_RUNNER" secret rm YUBIKEY_PIN 2>/dev/null || true; }
trap cleanup EXIT

"$CONTAINER_RUNNER" run --rm -it -q \
    --device "$YUBIKEY_PATH" \
    --secret YUBIKEY_PIN,type=env \
    -v "$SCRIPT_DIR/wait-for-pcscd.sh:/wait-for-pcscd.sh:Z" \
    -v "$SCRIPT_DIR/sign.sh:/sign.sh:Z" \
    -v "$SIGN_DIR:/sign_dir:Z" \
    "${optional_override_provider_config[@]}" \
    -w "/sign_dir" \
    --entrypoint /wait-for-pcscd.sh \
    "$CONTAINER_IMAGE_NAME" \
    bash -c "$2"
