#!/usr/bin/env bash

# Given a directory will sign all artifacts (apk and aab files) in that directory.
# Requires a YUBIKEY PIN and credentials directory to be set up beforehand.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="dist/fdroid"
METADATA_FILE="$REPO_DIR/metadata/net.mullvad.mullvadvpn.yml"
CONFIG_FILE="$SCRIPT_DIR/config.yml"
PROVIDER_ARG="${PROVIDER_ARG:-/usr/local/etc/pkcs11_java.cfg}"
KEY_ALIAS="X.509 Certificate for PIV Authentication"
cd "$SCRIPT_DIR"

function print_usage {
    echo "Usage: fdroid-deploy <option>"
    echo "Where option is one of the following flags:"
    echo "    -u, --update"
    echo "              Update the repo with the given version name and code"
    echo "    -s, --sign"
    echo "              Sign the index and entry files"
    echo "    -d, --deploy"
    echo "              Deploy the fdroid repo to the given folder"
    echo "    -h, --help"
    echo "              Show this help page."
}

function main {
    if [[ $# -eq 0 ]]; then
        print_usage
        exit 1
    fi

    case "$1" in
    "-u"|"--update")
        setup_repo "$2"
        fdroid update --nosign
        ;;
    "-s"|"--sign")

        if [[ -z ${YUBIKEY_PIN-} ]]; then
            echo "YUBIKEY_PIN pin must be set."
            exit 1
        fi

        sign
        ;;
    "-h"|"--help")
        print_usage
        exit 0
        ;;
    *)
        echo "Invalid argument: \`$1\`"
        print_usage
        exit 1
        ;;
    esac
}

function sign {
    # This is an approximation of fdroid signindex
    pushd "repo"

    # Sign and rename index_unsigned.jar
    local index_unsigned_jar="index_unsigned.jar"
    local signed_index_jar="index.jar"
    jarsigner_sign "$index_unsigned_jar"
    mv "$index_unsigned_jar" "$signed_index_jar"
    echo "Unsigned index jar signed"

    # Place index-v1 in a jar and sign
    local index_v1_json="index-v1.json"
    local index_v1_jar="index-v1.jar"
    zip -r "$index_v1_jar" "$index_v1_json"
    jarsigner_sign "$index_v1_jar"
    echo "Index v1 jar signed"

    # Place entry.json in a jar and sign
    # This uses apksigner as that is what fdroid does, unclear if we actually need to
    local entry_json="entry.json"
    local entry_jar="entry.jar"
    zip -r "$entry_jar" "$entry_json"
    apksigner_sign "$entry_jar"
    echo "Entry jar signed"

    popd
}

function apksigner_sign {
    local file="$1"

    apksigner -J-add-exports="jdk.crypto.cryptoki/sun.security.pkcs11=ALL-UNNAMED" sign \
    --ks NONE --ks-type PKCS11 --ks-key-alias "$KEY_ALIAS" \
    --provider-class sun.security.pkcs11.SunPKCS11 --provider-arg "$PROVIDER_ARG" \
    --min-sdk-version 23 --max-sdk-version 24 \
    --v4-signing-enabled false --v3-signing-enabled false --v2-signing-enabled false --v1-signing-enabled true \
    --in "$file" <<< "$YUBIKEY_PIN"
}

function jarsigner_sign {
    local file="$1"

    jarsigner -keystore NONE --store-type PKCS11  \
    -providerClass sun.security.pkcs11.SunPKCS11 \
    --providerArg "$PROVIDER_ARG" "$file" "$KEY_ALIAS" <<< "$YUBIKEY_PIN"
}

function setup_repo {
    if (( $# < 1 )); then
        echo "Provide the path to an apk file" >&2
        exit 1
    fi

    local apk="/build/$1"

    local version_code=$(apkanalyzer manifest version-code "$apk")

    # Create repo folder if needed
    mkdir -p "/build/$REPO_DIR"

    # Copy the config file if required
    if [ ! -f "/build/$REPO_DIR/config.yml" ]; then
        cp "config.yml" "/build/$REPO_DIR/config.yml"
    fi

    # Copy the metadata file if required
    if [ ! -f "/build/$METADATA_FILE" ]; then
        mkdir -p "/build/$REPO_DIR/metadata"
        cp "net.mullvad.mullvadvpn.yml" "/build/$METADATA_FILE"
    fi

    # Copy the icon file if required
    if [ ! -f "/build/$REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/images/icon.png" ]; then
        mkdir -p "/build/$REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/images"
        cp "icon.png" \
        "/build/$REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/images/icon.png"
    fi

    # Copy the apk file into the repo
    cp "$apk" "repo/net.mullvad.mullvadvpn_$version_code.apk"

    # Copy the release notes into the repo
    mkdir -p "/build/$REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/changelogs"
    cp "../../../../android/src/main/play/release-notes/en-US/default.txt" \
    "/build/$REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/changelogs/${version_code}.txt"
}

main "$@"
