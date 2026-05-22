#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}
CONTAINER_IMAGE_NAME=$(cat "$SCRIPT_DIR/../../building/android-container-image.txt")

WORK_DIR=${1:?'Usage: containerized-sign.sh <work-dir> <bash-command>'}
# Default to original sign command for backwards compatibility.
COMMAND=${2:-'shopt -s nullglob; /sign.sh MullvadVPN-*.aab MullvadVPN-*.apk'}

if [[ ! -d "$WORK_DIR" ]]; then
    echo "Error: not a directory: $WORK_DIR"
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
    -v "$WORK_DIR:/work:Z" \
    "${optional_override_provider_config[@]}" \
    -w "/work" \
    --entrypoint /wait-for-pcscd.sh \
    "$CONTAINER_IMAGE_NAME" \
    bash -c "$COMMAND"
