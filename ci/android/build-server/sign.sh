#!/usr/bin/env bash

# Given a directory will sign all Mullvad apk and aab files in that directory.
# APKs will be signed with both the old and the new key. AABs will only be signed by the new key.
# Requires a YUBIKEY PIN and credentials directory to be set up beforehand.

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
APKSIGNER_CMD="${APKSIGNER_CMD:-apksigner}"
PROVIDER_ARG="$BUILD_DIR/ci/android/build-server/signing/pkcs11_java.cfg"
KEY_ALIAS="X.509 Certificate for PIV Authentication"

if [[ -z ${YUBIKEY_PIN-} ]]; then
    echo "Needs to have set yubikey pin"
    exit 1
fi

function main {
    if [[ $# -eq 0 ]]; then
        echo "Please specify which folder to sign files in"
        exit 1
    fi

    if [[ $# -gt 1 ]]; then
        echo "Too many arguments"
        exit 1
    fi

    sign_artifacts "$1"
}

function sign_artifact {
    local file=$1

    echo "$YUBIKEY_PIN" | $APKSIGNER_CMD -J-add-exports="jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED" sign \
    --ks NONE --ks-type PKCS11 --ks-key-alias "$KEY_ALIAS" \
    --provider-class sun.security.pkcs11.SunPKCS11 --provider-arg "$PROVIDER_ARG" \
    --min-sdk-version 26 \
    --in "$file"
}

function sign_artifacts {
    dir="$1"

    pushd "$dir"
    # Sign all Mullvad apk and aab files
    for apk in MullvadVPN-*.apk MullvadVPN-*.aab; do
        sign_artifact "$apk"
    done
    popd
}

# Run script
main "$@"
