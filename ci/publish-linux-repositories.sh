#!/usr/bin/env bash
#
# Usage: ./publish-linux-repositories.sh [--production/--staging/--dev] <artifact dir> <app version>
#
# Copies app deb and rpm artifacts over to the repository building service inbox directory.
# Makes that service publish the new artifacts to the corresponding repository server.

set -eu
# nullglob is needed to produce expected results when globing an empty directory
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# shellcheck source=ci/buildserver-config.sh
source "$SCRIPT_DIR/buildserver-config.sh"

while [ "$#" -gt 0 ]; do
    case "$1" in
        "--production")
            environment="production"
            ;;
        "--staging")
            environment="staging"
            ;;
        "--dev")
            environment="dev"
            ;;
        -*)
            echo "Unknown option \"$1\"" >&2
            exit 1
            ;;
        *)
            if [[ -z ${artifact_dir+x} ]]; then
                artifact_dir=$1
            elif [[ -z ${version+x} ]]; then
                version=$1
            else
                echo "Too many arguments" >&2
                exit 1
            fi
            ;;
    esac
    shift
done

if [[ -z ${environment+x} ]]; then
    echo "Pass either --dev, --staging or --production to select target servers" >&2
    exit 1
fi
if [[ -z ${artifact_dir+x} ]]; then
    echo "Please give the artifact directory as an argument to this script" >&2
    exit 1
fi
if [[ -z ${version+x} ]]; then
    echo "Please give the release version as an argument to this script" >&2
    exit 1
fi

function copy_linux_artifacts_to_dir {
    local src_dir=$1
    local version=$2
    local dst_dir=$3

    for deb_path in "$src_dir"/MullvadVPN-"$version"*.deb; do
        cp "$deb_path" "$dst_dir/"
    done
    for rpm_path in "$src_dir"/MullvadVPN-"$version"*.rpm; do
        cp "$rpm_path" "$dst_dir/"
    done
}

function notify_repository_service {
    local artifact_dir=$1
    local version=$2
    local repository_inbox_dir=$3

    local tmp_notify_file
    tmp_notify_file=$(mktemp -p "$LINUX_REPOSITORY_NOTIFY_FILE_TMP_DIR")
    local notify_file="$repository_inbox_dir/app.src"

    # Temporarily write the file to a different path and then move it.
    # As long as the tmp dir and destination dir is on the same filesystem,
    # this is guaranteed to be atomic, preventing partial reads by the consuming
    # repository building service.
    echo "$artifact_dir" > "$tmp_notify_file"
    echo "DEBUG: Moving notify file $tmp_notify_file -> $notify_file"
    mv "$tmp_notify_file" "$notify_file"
}

stable_or_beta="stable"
if [[ $version == *"-beta"* ]]; then
    stable_or_beta="beta"
fi
repository_inbox_dir="$LINUX_REPOSITORY_INBOX_DIR_BASE/$environment/$stable_or_beta"
repository_tmp_store_dir="$LINUX_REPOSITORY_TMP_STORE_DIR_BASE/$environment/$stable_or_beta/$version"

echo "Copying app artifacts for $version from $artifact_dir to $repository_tmp_store_dir"
rm -rf "$repository_tmp_store_dir" && mkdir -p "$repository_tmp_store_dir" || exit 1
copy_linux_artifacts_to_dir "$artifact_dir" "$version" "$repository_tmp_store_dir"

echo "Notifying repository building service in $repository_inbox_dir"
notify_repository_service "$repository_tmp_store_dir" "$version" "$repository_inbox_dir"
