#!/usr/bin/env bash

# Given a directory will sign all artifacts (apk and aab files) in that directory.
# Requires a YUBIKEY PIN and credentials directory to be set up beforehand.

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROVIDER_ARG="$SCRIPT_DIR/signing/pkcs11_java.cfg"
APKSIGNER_CMD="${APKSIGNER_CMD:-apksigner}"
KEY_ALIAS="X.509 Certificate for PIV Authentication"
MIN_SDK_VERSION="28"

if [[ -z ${YUBIKEY_PIN-} ]]; then
    echo "YUBIKEY_PIN pin must be set."
    exit 1
fi

function main {
    if [[ $# -eq 0 ]]; then
        echo "Please specify which files to sign"
        exit 1
    fi

    for artifact_file in "$@"; do
        sign_artifact "$artifact_file"
    done
}

function sign_artifact {
    local artifact_file="$1"

    $APKSIGNER_CMD -J-add-exports="jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED" sign \
    --ks NONE --ks-type PKCS11 --ks-key-alias "$KEY_ALIAS" \
    --provider-class sun.security.pkcs11.SunPKCS11 --provider-arg "$PROVIDER_ARG" \
    --min-sdk-version "$MIN_SDK_VERSION" --v4-signing-enabled false \
    --in "$artifact_file" <<< "$YUBIKEY_PIN"
}

main "$@"
