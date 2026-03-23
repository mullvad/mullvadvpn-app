#!/usr/bin/env bash

# Given a directory will sign all artifacts (apk and aab files) in that directory.
# Requires a YUBIKEY PIN and credentials directory to be set up beforehand.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
METADATA_FILE="$SCRIPT_DIR/metadata/net.mullvad.mullvadvpn.yml"
CONFIG_FILE="$SCRIPT_DIR/config.yml"
UPLOAD_DIR="/upload"

if [[ -z ${YUBIKEY_PIN-} ]]; then
    echo "YUBIKEY_PIN pin must be set."
    exit 1
fi

function main {
    if (( $# < 2 )); then
        echo "Provide a version name and a version code" >&2
        exit 1
    fi
   
    setup_repo "$1" "$2"

    fdroid update

    fdroid publish
}

function setup_repo {
    local version_name="$1"
    local version_code="$2"

    # Install fdroid server
    apt update
    apt install -y fdroidserver

    # Replace version name and version code
    sed "s/^CurrentVersion: .*/CurrentVersion: ${version_name}/" "$METADATA_FILE" > \
    /tmp/tmpfile && mv /tmp/tmpfile "$METADATA_FILE"
    sed "s/^versionName: .*/versionName: ${version_name}/" "$METADATA_FILE" > \
    /tmp/tmpfile && mv /tmp/tmpfile "$METADATA_FILE"

    sed "s/^CurrentVersionCode: .*/CurrentVersionCode: ${version_code}/" "$METADATA_FILE" > \
    /tmp/tmpfile && mv /tmp/tmpfile "$METADATA_FILE"
    sed "s/^versionCode: .*/versionCode: ${version_code}/" "$METADATA_FILE" > \
    /tmp/tmpfile && mv /tmp/tmpfile "$METADATA_FILE"

    # Set upload dir in the config file
    sed "s|^local_copy_dir: .*|local_copy_dir: ${UPLOAD_DIR}|" \
    "$CONFIG_FILE" > /tmp/tmpfile && mv /tmp/tmpfile "$CONFIG_FILE"
}

main "$@"
