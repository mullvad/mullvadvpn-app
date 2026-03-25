#!/usr/bin/env bash

# Given a directory will sign all Mullvad apk and aab files in that directory.
# APKs will be signed with both the old and the new key. AABs will only be signed by the new key.
# Requires a YUBIKEY PIN and credentials directory to be set up beforehand.

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
APKSIGNER_CMD="${APKSIGNER_CMD:-apksigner}"
SIGNING_CERTIFICATE_LINEAGE="$BUILD_DIR/ci/android/build-server/signing/SigningCertificateLineage"
PROVIDER_ARG="$BUILD_DIR/ci/android/build-server/signing/provider-arg.cfg"
KEY_ALIAS="Certificate for PIV Authentication"

if [[ -z ${YUBIKEY_PIN-} ]]; then
    echo "Needs to have set yubikey pin"
    exit 1
fi

if [[ -z ${ANDROID_CREDENTIALS_DIR-} || ! -d "$ANDROID_CREDENTIALS_DIR" || -z "$(ls -A "$ANDROID_CREDENTIALS_DIR")" ]]; then
    echo "Credentials directory is missing or empty"
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
    local rotate=${2:-false}

    if $rotate; then
        echo "$YUBIKEY_PIN" | $APKSIGNER_CMD -J-add-exports="jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED" sign \
        --ks "$ANDROID_CREDENTIALS_DIR/app-keys.jks" \
        --ks-pass "file:$ANDROID_CREDENTIALS_DIR/keystore.properties.new" \
        --next-signer --ks NONE --ks-type PKCS11 --ks-key-alias "$KEY_ALIAS" \
        --provider-class sun.security.pkcs11.SunPKCS11 --provider-arg "$PROVIDER_ARG" \
        --lineage "$SIGNING_CERTIFICATE_LINEAGE" --rotation-min-sdk-version 28 --in "$file"
    else
        echo "$YUBIKEY_PIN" | $APKSIGNER_CMD -J-add-exports="jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED" sign \
        --ks NONE --ks-type PKCS11 --ks-key-alias "$KEY_ALIAS" \
        --provider-class sun.security.pkcs11.SunPKCS11 --provider-arg "$PROVIDER_ARG" \
        --in "$file"
    fi
}

function sign_artifacts {
    dir="$1"

    pushd "$dir"
    # Sign all apk files with the old and new key
    for apk in MullvadVPN-*.apk; do
        sign_artifact "$apk" true
    done

    # Sign all aab files with the upload key (new key)
    for aab in MullvadVPN-*.aab
    do
        sign_artifact "$aab" false
    done
    popd
}

# Run script
main "$@"
