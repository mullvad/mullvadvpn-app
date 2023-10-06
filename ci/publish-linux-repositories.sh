#!/usr/bin/env bash
#
# Usage: ./publish_linux_repositories.sh <app version>
#
# Rsyncs a locally prepared and stored DEB repository to production repository servers

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

LINUX_REPOSITORY_SERVERS=("se-got-cdn-111.mullvad.net" "se-mma-cdn-101.mullvad.net")

version=$1
deb_repo_dir="deb/$version"

function rsync_repo {
    local local_repo_dir=$1
    local remote_repo_dir=$2

    for server in "${LINUX_REPOSITORY_SERVERS[@]}"; do
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

read -rp "Do you want to remove the repository locally at $deb_repo_dir? (y/n) "
if [[ "$REPLY" =~ [Yy]$ ]]; then
    rm -r "$deb_repo_dir"
fi
