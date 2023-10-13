#!/usr/bin/env bash
#
# Usage: ./publish-linux-repositories.sh [--production/--staging] <app version> <deb repository dir>
#
# Rsyncs a locally prepared and stored apt repository to the dev/staging/production
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
            echo "Unknown option \"$1\"" >&2
            exit 1
            ;;
        *)
            if [[ -z ${version+x} ]]; then
                version=$1
            elif [[ -z ${deb_repo_dir+x} ]]; then
                deb_repo_dir=$1
            else
                echo "Too many arguments" >&2
                exit 1
            fi
            ;;
    esac
    shift
done

if [[ -z ${version+x} ]]; then
    echo "Please give the release version as an argument to this script" >&2
    exit 1
fi
if [[ -z ${deb_repo_dir+x} ]]; then
    echo "Please specify the deb repository directory as an argument to this script" >&2
    exit 1
fi
if [[ -z ${repository_servers+x} ]]; then
    echo "Pass either --dev, --staging or --production to select target servers" >&2
    exit 1
fi

function rsync_repo {
    local local_repo_dir=$1
    local remote_repo_dir=$2

    for server in "${repository_servers[@]}"; do
        echo "Syncing to $server:$remote_repo_dir"
        rsync -av --delete --mkpath --rsh='ssh -p 1122' \
            "$local_repo_dir"/ \
            build@"$server":"$remote_repo_dir"
    done
}

if [[ ! -d "$deb_repo_dir" ]]; then
    echo "$deb_repo_dir does not exist" >&2
    exit 1
fi

rsync_repo "$deb_repo_dir" "deb/beta"
if [[ $version != *"-beta"* ]]; then
    rsync_repo "$deb_repo_dir" "deb/stable"
fi
