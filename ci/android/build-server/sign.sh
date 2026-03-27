#!/usr/bin/env bash

# Given a directory will sign all artifacts (apk and aab files) in that directory.
# Requires a YUBIKEY PIN and credentials directory to be set up beforehand.

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
PROVIDER_ARG="$BUILD_DIR/ci/android/build-server/signing/pkcs11_java.cfg"
APKSIGNER_CMD="${APKSIGNER_CMD:-apksigner}"
KEY_ALIAS="X.509 Certificate for PIV Authentication"
MIN_SDK_VERSION="26"

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

    local artifact_dir="$1"
    sign_artifacts "$artifact_dir"
}

function sign_artifacts {
    local artifact_dir="$1"

    pushd "$artifact_dir"
    for artifact in MullvadVPN-*.apk MullvadVPN-*.aab; do
        sign_artifact "$artifact"
    done
    popd
}

function sign_artifact {
    local artifact="$1"

    echo "$YUBIKEY_PIN" | $APKSIGNER_CMD -J-add-exports="jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED" sign \
    --ks NONE --ks-type PKCS11 --ks-key-alias "$KEY_ALIAS" \
    --provider-class sun.security.pkcs11.SunPKCS11 --provider-arg "$PROVIDER_ARG" \
    --min-sdk-version "$MIN_SDK_VERSION" \
    --in "$artifact"
}

main "$@"
