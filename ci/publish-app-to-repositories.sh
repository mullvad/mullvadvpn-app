#!/usr/bin/env bash

set -eu
# nullglob is needed to produce expected results when globing an empty directory
shopt -s nullglob

function usage() {
    echo "Usage: $0 [--production/--staging/--dev] <artifact dir> <app version>"
    echo
    echo "Copies app deb and rpm artifacts over to the repository building service inbox directory."
    echo "Makes that service publish the new artifacts to the corresponding repository server."
    echo
    echo "Options:"
    echo "  -h | --help		Show this help message and exit."
    echo "  --production    Publish app to production environment"
    echo "  --staging       Publish app to staging environment"
    echo "  --dev           Publish app to development environment"
    echo ""
    echo "Arguments:"
    echo "  <artifact dir>  Directory to copy from. Will copy all app deb/rpms matching <version>"
    echo "  <version>       App version to copy. If <artifact dir> has apps for multiple versions"
    echo "                  only apps matching this version will be copied"
    exit 1
}

if [[ "$#" == 0 || $1 == "-h" || $1 == "--help" ]]; then
    usage
fi

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
        echo "Copying $deb_path into $dst_dir/"
        cp "$deb_path" "$dst_dir/"
    done
    for rpm_path in "$src_dir"/MullvadVPN-"$version"*.rpm; do
        echo "Copying $rpm_path into $dst_dir/"
        cp "$rpm_path" "$dst_dir/"
    done
}

function notify_repository_service {
    local artifact_dir=$1
    local version=$2
    local repository_inbox_dir=$3

    local tmp_notify_file
    tmp_notify_file=$(mktemp -p "$repository_inbox_dir")
    local notify_file="$repository_inbox_dir/app.src"

    # Temporarily write the file to a different path and then move it.
    # As long as the tmp dir and destination dir is on the same filesystem,
    # this is guaranteed to be atomic, preventing partial reads by the consuming
    # repository building service.
    echo "$artifact_dir" > "$tmp_notify_file"
    echo "Writing notify file $notify_file"
    mv "$tmp_notify_file" "$notify_file"
}

function publish_app_to_repo {
    # source files
    local artifact_dir=$1
    local version=$2
    # destination repository
    local environment=$3
    local stable_or_beta=$4

    echo "[#] Publishing $version to $environment/$stable_or_beta"

    local repository_inbox_dir="$LINUX_REPOSITORY_INBOX_DIR_BASE/$environment/$stable_or_beta"
    local repository_tmp_store_dir
    repository_tmp_store_dir="$(mktemp -qdt "mullvadvpn-app-$version-tmp-XXXXXXX")"

    echo "Copying app artifacts for $version from $artifact_dir to $repository_tmp_store_dir"
    copy_linux_artifacts_to_dir "$artifact_dir" "$version" "$repository_tmp_store_dir"

    echo "Notifying repository building service in $repository_inbox_dir"
    notify_repository_service "$repository_tmp_store_dir" "$version" "$repository_inbox_dir"
}

publish_app_to_repo "$artifact_dir" "$version" "$environment" "beta"
if [[ $version != *"-beta"* ]]; then
    publish_app_to_repo "$artifact_dir" "$version" "$environment" "stable"
fi
