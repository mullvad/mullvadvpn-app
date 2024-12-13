#!/usr/bin/env bash

set -eu
# nullglob is needed to produce expected results when globing an empty directory
shopt -s nullglob

function usage() {
    echo "Usage: $0 <repository dir> <repository server url> <remote repo dir> <stable_or_beta> <artifact dirs...>"
    echo
    echo "Example: $0 ./repos/rpm https://cdn.mullvad.net/ rpm/stable stable inbox/app.latest inbox/browser.latest"
    echo
    echo "Will create an rpm repository in <repository dir> and add all .rpm files from all <artifact dirs> to it."
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
repository_server_url=${2:?"Give repository server URL as second argument (needed for repository metadata generation)"}
remote_repo_dir=${3:?"Give remote repo dir as third argument"}
stable_or_beta=${4:?"Give 'stable' or 'beta' as fourth argument"}
shift 4

artifact_dirs=()
while [ "$#" -gt 0 ]; do
    artifact_dirs+=("$1")
    shift
done

if [[ "$stable_or_beta" != "beta" && "$stable_or_beta" != "stable" ]]; then
    echo "<stable or beta> argument must be 'stable' or 'beta'"
    exit 1
fi
if [ "${#artifact_dirs[@]}" -lt 1 ]; then
    echo "No artifact directories given" >&2
    exit 1
fi

# Writes the mullvad.repo config file to the repository root.
# This needs to contain the absolute url and path to the repository.
# As such, it depends on what server we upload to as well as if it's stable or beta.
function generate_rpm_repository_configuration {
    local repository_dir=$1
    local repository_server_url=$2
    local remote_repo_dir=$3
    local stable_or_beta=$4

    local repository_name="Mullvad VPN"
    if [[ "$stable_or_beta" == "beta" ]]; then
        repository_name+=" (beta)"
    fi

    echo -e "[mullvad-$stable_or_beta]
name=$repository_name
baseurl=$repository_server_url/$remote_repo_dir/\$basearch
type=rpm
enabled=1
gpgcheck=1
gpgkey=$repository_server_url/rpm/mullvad-keyring.asc
includepkgs=mullvad-vpn,mullvad-browser" > "$repository_dir/mullvad.repo"
}

for artifact_dir in "${artifact_dirs[@]}"; do
    for arch in "${SUPPORTED_RPM_ARCHITECTURES[@]}"; do
        arch_repo_dir="$repo_dir/$arch"
        for rpm_path in "$artifact_dir"/*"$arch".rpm; do
            mkdir -p "$arch_repo_dir"
            echo "[#] Copying $rpm_path to $arch_repo_dir/"
            cp "$rpm_path" "$arch_repo_dir"/
        done
    done
done

for arch in "${SUPPORTED_RPM_ARCHITECTURES[@]}"; do
    arch_repo_dir="$repo_dir/$arch"
    if [[ ! -d "$arch_repo_dir" ]]; then
        echo "No $arch repository in $repo_dir" >&2
        continue
    fi

    echo "Generating architecture repository metadata for $arch_repo_dir"

    # Generate repository metadata files (containing among other things checksums
    # for the rpm files in it)
    createrepo_c "$arch_repo_dir"

    # Sign repository metadata (created by createrepo_c above)
    gpg --detach-sign --armor "$arch_repo_dir/repodata/repomd.xml"
done

echo "Generating global repository configuration in $repo_dir"
generate_rpm_repository_configuration "$repo_dir" "$repository_server_url" "$remote_repo_dir" "$stable_or_beta"

