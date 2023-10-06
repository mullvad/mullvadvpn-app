#!/usr/bin/env bash
#
# Usage: ./publish-linux-repositories.sh <app version> [--production/--staging]
#
# Rsyncs a locally prepared and stored DEB repository to staging or production
# repository servers.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

source "$SCRIPT_DIR/buildserver-config.sh"

while [ "$#" -gt 0 ]; do
    case "$1" in
        "--production")
            repository_servers=("${PRODUCTION_LINUX_REPOSITORY_SERVERS[@]}")
            ;;
        "--staging")
            repository_servers=("${STAGING_LINUX_REPOSITORY_SERVERS[@]}")
            ;;
        "--dev")
            repository_servers=("${DEV_LINUX_REPOSITORY_SERVERS[@]}")
            ;;
        -*)
            echo "Unknown option \"$1\""
            exit 1
            ;;
        *)
            version=$1
            ;;
    esac
    shift
done

if [[ -z ${version+x} ]]; then
    echo "Please give the release version as an argument to this script"
    exit 1
fi
if [[ -z ${repository_servers+x} ]]; then
    echo "Pass either --staging or --production to select target servers"
    exit 1
fi

deb_repo_dir="$DEB_REPOSITORY_ARCHIVE_DIR/$version"

function rsync_repo {
    local local_repo_dir=$1
    local remote_repo_dir=$2

    for server in "${repository_servers[@]}"; do
        echo "Syncing to $server"
        rsync -av --delete --mkpath --rsh='ssh -p 1122' \
            "$local_repo_dir"/ \
            build@"$server":"$remote_repo_dir"
    done
}

if [[ ! -d "$deb_repo_dir" ]]; then
    echo "$version is not a version we have a repository for"
    exit 1
fi

echo "Uploading DEB repository to deb/beta"
rsync_repo "$deb_repo_dir" "deb/beta"
if [[ $version != *"-beta"* ]]; then
    echo "Uploading DEB repository to deb/stable"
    rsync_repo "$deb_repo_dir" "deb/stable"
fi
