#!/usr/bin/env bash
#
# Usage: ./publish-linux-repositories.sh [--production/--staging/--dev] <app version> <deb repository dir> <rpm repository dir>
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
            repository_server_url="$PRODUCTION_LINUX_REPOSITORY_PUBLIC_URL"
            ;;
        "--staging")
            repository_servers=("${STAGING_LINUX_REPOSITORY_SERVERS[@]}")
            repository_server_url="$STAGING_LINUX_REPOSITORY_PUBLIC_URL"
            ;;
        "--dev")
            repository_servers=("${DEV_LINUX_REPOSITORY_SERVERS[@]}")
            repository_server_url="$DEV_LINUX_REPOSITORY_PUBLIC_URL"
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
            elif [[ -z ${rpm_repo_dir+x} ]]; then
                rpm_repo_dir=$1
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
if [[ -z ${rpm_repo_dir+x} ]]; then
    echo "Please specify the rpm repository directory as an argument to this script" >&2
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

# Writes the mullvad.repo config file to the repository
# root. This needs to contain the absolute url and path
# to the repository. As such, it depends on what server
# we upload to as well as if it's stable or beta. That's
# why we need to do it just before upload.
function generate_rpm_repository_configuration {
    local repository_dir=$1
    local stable_or_beta=$2

    echo -e "[mullvad-rpm]
name=Mullvad VPN
baseurl=$repository_server_url/rpm/$stable_or_beta/\$basearch
type=rpm
enabled=1
gpgcheck=1
gpgkey=$repository_server_url/rpm/mullvad-keyring.asc" > "$repository_dir/mullvad.repo"
}

if [[ ! -d "$deb_repo_dir" ]]; then
    echo "$deb_repo_dir does not exist" >&2
    exit 1
fi
if [[ ! -d "$rpm_repo_dir" ]]; then
    echo "$rpm_repo_dir does not exist" >&2
    exit 1
fi

rsync_repo "$deb_repo_dir" "deb/beta"

generate_rpm_repository_configuration "$rpm_repo_dir" "beta"
rsync_repo "$rpm_repo_dir" "rpm/beta"

if [[ $version != *"-beta"* ]]; then
    rsync_repo "$deb_repo_dir" "deb/stable"

    generate_rpm_repository_configuration "$rpm_repo_dir" "stable"
    rsync_repo "$rpm_repo_dir" "rpm/stable"
fi

