#!/usr/bin/env bash
#
# Usage: ./prepare-rpm-repository.sh <artifact dir> <app version> <repository dir>
#
# Will create an rpm repository in <repository dir> and add all .rpm files from
# <artifact dir> matching <app version> to the repository.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

source "$SCRIPT_DIR/buildserver-config.sh"

artifact_dir=$1
version=$2
repo_dir=$3

function generate_repository_configuration {
    echo -e "[mullvad-rpm]
name=Mullvad VPN
baseurl=https://repository.mullvad.net/rpm/\$basearch
type=rpm
enabled=1
gpgcheck=1
gpgkey=https://repository.mullvad.net/rpm/mullvad-keyring.asc"
}

function create_repository {
    local arch_repo_dir=$1
    local rpm_path=$2

    mkdir -p "$arch_repo_dir"

    # Copy RPM file into repository
    cp "$rpm_path" "$arch_repo_dir"/

    # Generate repository metadata files (containing among other things checksums
    # for the above artifact)
    createrepo_c "$arch_repo_dir"

    # Sign repository metadata (created by createrepo_c above)
    # --yes is passed to automatically overwrite existing files
    # in the case where the build server re-builds something we already
    # have built.
    gpg --detach-sign --armor --yes "$arch_repo_dir/repodata/repomd.xml"
}

for arch in "${SUPPORTED_RPM_ARCHITECTURES[@]}"; do
    rpm_path="$artifact_dir"/MullvadVPN-"$version"_"$arch".rpm
    if [[ ! -e "$rpm_path" ]]; then
        echo "RPM at $rpm_path does not exist" >&2
        exit 1
    fi
    create_repository "$repo_dir/$arch" "$rpm_path"
done

generate_repository_configuration > "$repo_dir/mullvad.repo"
