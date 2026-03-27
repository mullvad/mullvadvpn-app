#!/usr/bin/env bash

# Given a directory and a version will upload aab files matching the flavor and version to Google Play.

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

if [[ -z ${PLAY_CREDENTIALS_PATH-} ]]; then
    echo "PLAY_CREDENTIALS_PATH must be set"
    exit 1
fi

function main {
    if [[ $# -eq 0 || $# -eq 1 ]]; then
        echo "Please specify a artifact directory and version"
        exit 1
    fi

    if [[ $# -gt 2 ]]; then
        echo "Too many arguments"
        exit 1
    fi

    local artifact_dir="$1"
    local version="$2"

    if [[ ! -d "$artifact_dir" || -z "$(ls -A "$artifact_dir")" ]]; then
        echo "Missing or empty artifact directory"
        exit 1
    fi

    upload_bundles "$artifact_dir" "$version"
}

# Upload bundle files to Google play.
# It has to be done for one bundle at a time due to how the publish task works,
# see the upload_bundle function for more information.
function upload_bundles {
    artifact_dir="$1"
    version="$2"

    local play_upload_dir="$artifact_dir/play_upload"
    mkdir -p "$play_upload_dir"

    trap 'rc=$?; (( rc != 0 )) && rm -rf "$play_upload_dir"' EXIT

    if [[ "$version" != *"-dev-"* ]]; then
            upload_bundle "$play_upload_dir" "publishPlayProdReleaseBundle" "$artifact_dir/MullvadVPN-$version.play.aab" 
        if [[ "$version" == *"-alpha"* ]]; then
            upload_bundle "$play_upload_dir" "publishPlayDevmoleReleaseBundle" "$artifact_dir/MullvadVPN-$version.play.devmole.aab"
            upload_bundle "$play_upload_dir" "publishPlayStagemoleReleaseBundle" "$artifact_dir/MullvadVPN-$version.play.stagemole.aab"
        fi
    fi
}

# Due to to the publish task only being able to upload all files in a folder
# we need to upload one bundle at a time by copying into an upload dir.
function upload_bundle {
    upload_dir=$1
    publish_task=$2
    bundle_file=$3

    rm -r "${upload_dir:?}/"* 2>/dev/null || true
    cp "$bundle_file" "$upload_dir/"

    PLAY_CREDENTIALS_PATH="$PLAY_CREDENTIALS_PATH" \
    "$SCRIPT_DIR/../../../building/container-run.sh" android ./android/gradlew -p android "$publish_task" --artifact-dir "../$upload_dir"
}

main "$@"
