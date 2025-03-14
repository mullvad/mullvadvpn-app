#!/usr/bin/env bash
#
# Builds linux deb and rpm repositories and uploads them to a repository server.
# One instance of this script targets *one* server environment.
# This means that if you want to publish to development, staging and production servers,
# you need to run three instances of this script.
#
# This script works on an $inbox_dir. In this directory it expects one directory per repository.
# For each repository it will read all files having the .src extension 
# These files are expected to contain a single line, a path to some directory where
# it can read new artifacts for that product.
# All deb and rpm files from that directory will be signed and moved over to a folder with
# the same name as the .src file, but with a .latest extension instead.
# So artifacts read from `app.src` will be moved to `app.latest/`.
#
# Then the deb and rpm repositories will be generated and all deb and rpm files in
# $inbox_dir/$repository/*.latest/ will be added to the repository. These repositories are then synced
# to `$repository_server_upload_domain respectively.

set -eu
# nullglob is needed to produce expected results when globing an empty directory
shopt -s nullglob

function usage() {
    echo "Usage: $0 <environment>"
    echo
    echo "Example usage: ./build-linux-repositories.sh production"
    echo
    echo "This script reads an inbox folder for the corresponding server environment."
    echo "It then generates and uploads new Linux repositories for all"
    echo "the latest artifacts"
    echo
    echo "Options:"
    echo "  -h | --help		Show this help message and exit."
    exit 1
}

if [[ "$#" == 0 || $1 == "-h" || $1 == "--help" ]]; then
    usage
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# shellcheck source=ci/linux-repository-builder/build-linux-repositories-config.sh
source "$SCRIPT_DIR/build-linux-repositories-config.sh"

environment="$1"
case "$environment" in
    "production")
        repository_server_upload_domain="$PRODUCTION_REPOSITORY_SERVER"
        repository_server_public_url="$PRODUCTION_LINUX_REPOSITORY_PUBLIC_URL"
        ;;
    "staging")
        repository_server_upload_domain="$STAGING_REPOSITORY_SERVER"
        repository_server_public_url="$STAGING_LINUX_REPOSITORY_PUBLIC_URL"
        ;;
    "dev")
        repository_server_upload_domain="$DEV_REPOSITORY_SERVER"
        repository_server_public_url="$DEV_LINUX_REPOSITORY_PUBLIC_URL"
        ;;
    *)
        echo "Unknown environment. Specify production, staging or dev" >&2
        exit 1
        ;;
esac

inbox_dir="$LINUX_REPOSITORY_INBOX_DIR_BASE/$environment"

if [[ ! -d "$inbox_dir" ]]; then
    echo "Inbox $inbox_dir does not exist" 1>&2
    exit 1
fi

# Process all .src files in the given inbox dir.
# Signs and moves all found artifacts into .latest directories
# Returns 0 if everything went well and there are new artifacts for a product.
# Returns 1 if no new artifacts were found
function process_inbox {
    local inbox_dir=$1
    echo "[#] Processing inbox at $inbox_dir"

    local found_new_artifacts="false"
    # Read all notify files and move the artifacts they point to into a local .latest copy
    for notify_file in "$inbox_dir"/*.src; do
        if [[ ! -f "$notify_file" ]]; then
            echo "Ignoring non-file $notify_file" 1>&2
            continue
        fi
        echo "Processing notify file $notify_file"

        local src_dir
        src_dir=$(cat "$notify_file")
        if [[ ! -d "$src_dir" ]]; then
            echo "Artifact source dir $src_dir from notify file $notify_file does not exist" 1>&2
            continue
        fi

        # Removing the file before we move the artifacts out of where it points.
        # This ensures we don't get stuck in a loop processing it over and over
        # if the signing and moving fails.
        rm "$notify_file"

        local artifact_dir=${notify_file/%.src/.latest}
        # Recreate the artifact dir, cleaning it
        rm -rf "$artifact_dir" && mkdir -p "$artifact_dir" || exit 1

        # The fact that we have processed one .src file is enough tro trigger a repository
        # rebuild. Because if a product would like to publish "no artifacts" they should
        # be able to create a .src file pointing to an empty directory
        found_new_artifacts="true"

        echo "Moving artifacts from $src_dir to $artifact_dir"
        # Move all deb and rpm files into the .latest dir
        for src_deb in "$src_dir"/*.deb; do
            echo "Signing and moving $src_deb into $artifact_dir/"
            dpkg-sig --sign builder "$src_deb"
            mv "$src_deb" "$artifact_dir/"
        done
        for src_rpm in "$src_dir"/*.rpm; do
            echo "Signing and moving $src_rpm into $artifact_dir/"
            rpm --addsign "$src_rpm"
            mv "$src_rpm" "$artifact_dir/"
        done
        rm -r "$src_dir" || echo "Failed to remove src dir $src_dir"
    done

    if [[ $found_new_artifacts == "false" ]]; then
        return 1
    fi
    return 0
}

function rsync_repo {
    local local_repo_dir=$1
    local remote_repo_dir=$2

    echo "Syncing to $repository_server_upload_domain:$remote_repo_dir"
    # We have an issue where the rsync can fail due to the remote dir being locked (only one rsync at a time allowed)
    # We suspect this is because of too fast subsequent invocations of rsync to the same target dir. With a hacky sleep
    # we hope to avoid this issue for now.
    sleep 10
    rsync -av --delete --mkpath --rsh='ssh -p 1122' \
        "$local_repo_dir"/ \
        build@"$repository_server_upload_domain":"$remote_repo_dir"
}

function invalidate_bunny_cdn_cache {
    curl --request POST \
        --url "https://api.bunny.net/pullzone/${BUNNYCDN_PULL_ZONE_ID}/purgeCache" \
        --header "AccessKey: ${BUNNYCDN_API_KEY}" \
        --header 'content-type: application/json' \
        --fail-with-body
}

for repository in "${REPOSITORIES[@]}"; do
    deb_remote_repo_dir="deb/$repository"
    rpm_remote_repo_dir="rpm/$repository"

    repository_inbox_dir="$inbox_dir/$repository"
    if ! process_inbox "$repository_inbox_dir"; then
        echo "Nothing new in inbox at $repository_inbox_dir"
        continue
    fi

    # Read all .latest artifact dirs into array
    readarray -d '' artifact_dirs < <(find "$repository_inbox_dir" -maxdepth 1 -name "*.latest" -type d -print0)
    if [ "${#artifact_dirs[@]}" -lt 1 ]; then
        echo "No artifact directories in $repository_inbox_dir to generate repositories from" >&2
        continue
    fi

    echo "Generating repositories from these artifact directories: ${artifact_dirs[*]}"

    # Generate deb repository from all the .latest artifacts

    deb_repo_dir="$repository_inbox_dir/repos/deb"
    rm -rf "$deb_repo_dir" && mkdir -p "$deb_repo_dir" || exit 1
    "$SCRIPT_DIR/prepare-apt-repository.sh" "$deb_repo_dir" "${artifact_dirs[@]}"

    # Generate rpm repository from all the .latest artifacts

    rpm_repo_dir="$repository_inbox_dir/repos/rpm"
    rm -rf "$rpm_repo_dir" && mkdir -p "$rpm_repo_dir" || exit 1
    "$SCRIPT_DIR/prepare-rpm-repository.sh" "$rpm_repo_dir" \
        "$repository_server_public_url" "$rpm_remote_repo_dir" "$repository" "${artifact_dirs[@]}"

    # rsync repositories to repository server

    echo "[#] Syncing deb repository to $deb_remote_repo_dir"
    rsync_repo "$deb_repo_dir" "$deb_remote_repo_dir"
    echo "[#] Syncing rpm repository to $rpm_remote_repo_dir"
    rsync_repo "$rpm_repo_dir" "$rpm_remote_repo_dir"

    if [[ "$environment" == "production" ]]; then
        echo "[#] Invalidating Bunny CDN cache"
        invalidate_bunny_cdn_cache
    fi

done
