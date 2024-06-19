#!/usr/bin/env bash
#
# Builds linux deb and rpm repositories and uploads them to a repository server.
# One instance of this script targets *one* server environment and *one* repository.
# This means that if you want to publish to development, staging and production servers,
# as well as publish both beta and stable repositories, you need to run six instances of
# this script.
#
# This script works on an $inbox_dir. In this directory it will read all files having the .src
# extension. These files are expected to contain a single line, a path to some directory where
# it can read new artifacts for that product.
# All deb and rpm files from that directory will be moved over to a folder with the same name
# as the .src file, but with a .latest extension instead. So artifacts read from `app.src`
# will be moved to `app.latest/`.
#
# Then the deb and rpm repositories will be generated and all deb and rpm files in
# $inbox_dir/*.latest/ will be added to the repository. These repositories are then synced
# to `$repository_server_upload_domain:$*_remote_repo_dir` respectively.

set -eu
# nullglob is needed to produce expected results when globing an empty directory
shopt -s nullglob

function usage() {
    echo "Usage: $0 <environment>-<repository>"
    echo
    echo "Example usage: ./build-linux-repositories.sh production-beta"
    echo
    echo "This script reads an inbox folder for the corresponding server environment and"
    echo "repository name. It then generates and uploads new Linux repositories for all"
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

# shellcheck source=ci/build-linux-repositories-config.sh
source "$SCRIPT_DIR/build-linux-repositories-config.sh"

# Split first argument at '-' and treat the first part as environment and second as repository
IFS='-' read -ra args <<< "$1"
environment=${args[0]:?"Give environment as argument before dash: production, staging or dev"}
stable_or_beta=${args[1]:?"Give stable or beta as argument after dash"}

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
if [[ "$stable_or_beta" != "beta" && "$stable_or_beta" != "stable" ]]; then
    echo "Unknown repository. Specify 'stable' or 'beta'"
    exit 1
fi

inbox_dir="$LINUX_REPOSITORY_INBOX_DIR_BASE/$environment/$stable_or_beta"
deb_remote_repo_dir="deb/$stable_or_beta"
rpm_remote_repo_dir="rpm/$stable_or_beta"

echo "DEBUG: inbox_dir: $inbox_dir"
echo "DEBUG: repository_server_upload_domain: $repository_server_upload_domain"
echo "DEBUG: repository_server_public_url: $repository_server_public_url"
echo "DEBUG: deb_remote_repo_dir: $deb_remote_repo_dir"
echo "DEBUG: rpm_remote_repo_dir: $rpm_remote_repo_dir"

found_new_artifacts="false"
# Read all notify files and move the artifacts they point to into a local .latest copy
for notify_file in "$inbox_dir"/*.src; do
    echo "DEBUG: notify_file=$notify_file"
    if [[ ! -f "$notify_file" ]]; then
        echo "Ignoring non-file $notify_file" 1>&2
        continue
    fi
    src_dir=$(cat "$notify_file")
    if [[ ! -d "$src_dir" ]]; then
        echo "Artifact source dir $src_dir from notify file $notify_file does not exist" 1>&2
        continue
    fi

    echo "DEBUG: Removing notify file $notify_file"
    rm "$notify_file"

    artifact_dir=${notify_file/%.src/.latest}
    # Recreate the artifact dir, cleaning it
    rm -rf "$artifact_dir" && mkdir -p "$artifact_dir" || exit 1

    echo "Moving artifacts from $src_dir to $artifact_dir"
    # Move all deb and rpm files into the .latest dir
    for src_deb in "$src_dir"/*.deb; do
        found_new_artifacts="true"
        echo "Moving $src_deb into $artifact_dir/"
        mv "$src_deb" "$artifact_dir/"
    done
    for src_rpm in "$src_dir"/*.rpm; do
        found_new_artifacts="true"
        echo "Moving $src_rpm into $artifact_dir/"
        mv "$src_rpm" "$artifact_dir/"
    done
    rm -r "$src_dir" || echo "Failed to remove src dir $src_dir"
done

if [[ $found_new_artifacts == "false" ]]; then
    echo "No new artifacts. Skipping repository generation and upload"
    exit 0
fi

# Read all .latest artifact dirs into array
readarray -d '' artifact_dirs < <(find "$inbox_dir" -maxdepth 1 -name "*.latest" -type d -print0)
if [ "${#artifact_dirs[@]}" -lt 1 ]; then
    echo "No artifact directories to generate repositories from" >&2
    exit 1
fi

echo "DEBUG: Generating repositories from these artifact directories: ${artifact_dirs[*]}"

# Generate deb repository from all the .latest artifacts

deb_repo_dir="$inbox_dir/repos/deb"
rm -rf "$deb_repo_dir" && mkdir -p "$deb_repo_dir" || exit 1
"$SCRIPT_DIR/prepare-apt-repository.sh" "$deb_repo_dir" "${artifact_dirs[@]}"

# Generate rpm repository from all the .latest artifacts

rpm_repo_dir="$inbox_dir/repos/rpm"
rm -rf "$rpm_repo_dir" && mkdir -p "$rpm_repo_dir" || exit 1
"$SCRIPT_DIR/prepare-rpm-repository.sh" "$rpm_repo_dir" "$repository_server_public_url" "$rpm_remote_repo_dir" "$stable_or_beta" "${artifact_dirs[@]}"

# rsync repositories to repository server

function rsync_repo {
    local local_repo_dir=$1
    local remote_repo_dir=$2

    echo "Syncing to $repository_server_upload_domain:$remote_repo_dir"
    rsync -av --delete --mkpath --rsh='ssh -p 1122' \
        "$local_repo_dir"/ \
        build@"$repository_server_upload_domain":"$remote_repo_dir"
}

rsync_repo "$deb_repo_dir" "$deb_remote_repo_dir"
rsync_repo "$rpm_repo_dir" "$rpm_remote_repo_dir"
