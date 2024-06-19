#!/usr/bin/env bash

set -eu
# nullglob is needed to produce expected results when globing an empty directory
shopt -s nullglob

function usage() {
    echo "Usage: $0 <repository dir> <artifact dirs...>"
    echo
    echo "Will create a deb repository in <repository dir> and add all .deb files from all <artifact dirs>"
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

repo_dir=${1:?"Specify the output repository directory as the first argument"}

artifact_dirs=()
while [ "$#" -gt 1 ]; do
    artifact_dirs+=("$2")
    shift
done

if [ "${#artifact_dirs[@]}" -lt 1 ]; then
    echo "No artifact directories given" >&2
    exit 1
fi

function generate_repository_configuration {
    local codename=$1

    echo -e "Origin: repository.mullvad.net
Label: Mullvad apt repository
Description: Mullvad package repository for Debian/Ubuntu
Codename: $codename
Architectures: amd64 arm64
Components: main
SignWith: $CODE_SIGNING_KEY_FINGERPRINT"
}

function generate_deb_distributions_content {
    local distributions=""
    for codename in "${SUPPORTED_DEB_CODENAMES[@]}"; do
        distributions+=$(generate_repository_configuration "$codename")$'\n'$'\n'
        distributions+=$(generate_repository_configuration "$codename"-testing)$'\n'$'\n'
    done
    echo "$distributions"
}

function add_deb_to_repo {
    local deb_path=$1
    local codename=$2
    echo "Adding $deb_path to repository $codename"
    reprepro -V --basedir "$repo_dir" --component main includedeb "$codename" "$deb_path"
}

echo "Generating deb repository into $repo_dir/"
mkdir -p "$repo_dir/conf"

echo "Writing repository configuration to $repo_dir/conf/distributions"
generate_deb_distributions_content > "$repo_dir/conf/distributions"
echo ""

for artifact_dir in "${artifact_dirs[@]}"; do
    for deb_path in "$artifact_dir"/*.deb; do
        for codename in "${SUPPORTED_DEB_CODENAMES[@]}"; do
            add_deb_to_repo "$deb_path" "$codename"
            echo ""
        done
        echo ""
    done
done
