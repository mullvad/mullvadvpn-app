#!/usr/bin/env bash

# Given a directory and a version will upload aab files matching the flavor and version to Google Play.

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"

if [[ -z ${ANDROID_CREDENTIALS_DIR-} || ! -d "$ANDROID_CREDENTIALS_DIR" || -z "$(ls -A "$ANDROID_CREDENTIALS_DIR")" ]]; then
    echo "Credentials directory is missing or empty"
    exit 1
fi

function main {
    if [[ $# -eq 0 || $# -eq 1 ]]; then
        echo "Please specify a folder and a version"
        exit 1
    fi

    if [[ $# -gt 2 ]]; then
        echo "Too many arguments"
        exit 1
    fi

    prepare "$1" "$2"
}

# Prepare files to be uploaded to Google play.
# Due to to the upload task only being able to upload all files in a folder we need to copy
# the file to a specific folder every time
function prepare {
    artifact_dir=$1
    version=$2

    local play_upload_dir="$artifact_dir/play_upload"
    mkdir -p "$play_upload_dir"

    trap 'rc=$?; (( rc != 0 )) && rm -rf "$play_upload_dir"' EXIT

    if [[ "$version" != *"-dev-"* ]]; then
            upload_google_play "publishPlayProdReleaseBundle" "$artifact_dir/MullvadVPN-$version.play.aab" "$play_upload_dir"
        if [[ "$version" == *"-alpha"* ]]; then
            upload_google_play "publishPlayDevmoleReleaseBundle" "$artifact_dir/MullvadVPN-$version.play.devmole.aab" "$play_upload_dir"
            upload_google_play "publishPlayStagemoleReleaseBundle" "$artifact_dir/MullvadVPN-$version.play.stagemole.aab" "$play_upload_dir"
        fi
    fi
}

function upload_google_play {
    task=$1
    file=$2
    upload_dir=$3

    rm -r "${upload_dir:?}/"* 2>/dev/null || true
    cp "$file" "$upload_dir/"

    PLAY_CREDENTIALS_PATH="$ANDROID_CREDENTIALS_DIR/play-api-key.json" \
    ./building/container-run.sh android ./android/gradlew -p android "$task" --artifact-dir "../$upload_dir"
}

# Run script
main "$@"
