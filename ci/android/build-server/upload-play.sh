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
        echo "Please specify an artifact directory and version"
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
    local artifact_dir=$1
    local version=$2

    if [[ "$version" != *"-dev-"* ]]; then
            upload_bundle "$artifact_dir" "$version" "play" "publishPlayProdReleaseBundle"
        if [[ "$version" == *"-alpha"* ]]; then
            upload_bundle "$artifact_dir" "$version" "play.devmole" "publishPlayDevmoleReleaseBundle"
            upload_bundle "$artifact_dir" "$version" "play.stagemole" "publishPlayStagemoleReleaseBundle"
        fi
    fi
}

# Due to to the publish task only being able to upload all files in a folder
# we need to upload one bundle at a time by copying into an upload dir.
function upload_bundle {
    local artifact_dir=$1
    local version=$2
    local artifact_suffix=$3
    local publish_task=$4

    local bundle_name="MullvadVPN-$version.$artifact_suffix.aab"
    local upload_dir="$artifact_dir/MullvadVPN-$version.$artifact_suffix.upload"

    mkdir "$upload_dir"
    cp "$artifact_dir/$bundle_name" "$upload_dir"

    PLAY_CREDENTIALS_PATH="$PLAY_CREDENTIALS_PATH" \
    "$SCRIPT_DIR/mullvadvpn-app/building/container-run.sh" android ./android/gradlew -p android "$publish_task" --artifact-dir "../$upload_dir"
}

main "$@"
