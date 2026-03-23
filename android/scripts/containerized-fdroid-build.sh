#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}
CONTAINER_IMAGE_NAME=$(cat "$SCRIPT_DIR/../../building/android-container-image.txt")
RELEASE_NOTES="$SCRIPT_DIR/../src/main/play/release-notes/en-US/default.txt"
FDROID_REPO_DIR="$SCRIPT_DIR/../fdroid-repo"

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

if (( $# < 4 )); then
        echo "A version name, version code,an apk and an upload directory needs to be provided" >&2
        exit 1
fi

version_name="$1"
version_code="$2"

# Copy the provided apk into fdroid repo
cp "$3" "$FDROID_REPO_DIR/repo"

upload_directory="$4"

printf '%s' "$YUBIKEY_PIN" | "$CONTAINER_RUNNER" secret create --replace YUBIKEY_PIN -
cleanup() { "$CONTAINER_RUNNER" secret rm YUBIKEY_PIN 2>/dev/null || true; }
trap cleanup EXIT

"$CONTAINER_RUNNER" run --rm -it -q \
    --device "$YUBIKEY_PATH" \
    --secret YUBIKEY_PIN,type=env \
    -v "$SCRIPT_DIR/wait-for-pcscd.sh:/wait-for-pcscd.sh:Z" \
    -v "$FDROID_REPO_DIR:/fdroid_repo_dir:Z" \
    -v "$RELEASE_NOTES:/fdroid_repo_dir/metadata/net.mullvad.mullvadvpn/en-US/changelogs/$version_code.txt:Z" \
    -v "$upload_directory:/upload:Z" \
    "${optional_override_provider_config[@]}" \
    -w "/fdroid_repo_dir" \
    --entrypoint /wait-for-pcscd.sh \
    "$CONTAINER_IMAGE_NAME" \
    bash -c "fdroid-deploy.sh $version_name $version_code"
