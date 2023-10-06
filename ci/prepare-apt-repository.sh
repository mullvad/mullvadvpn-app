#!/usr/bin/env bash
#
# Usage: ./prepare-apt-repository.sh <artifact dir> <app version> <repo dir>
#
# Will create a deb repository in <repo_dir> and add all .deb files from <artifact dir> matching
# <app version> to the repository.

set -eu

artifact_dir=$1
version=$2
repo_dir=$3

# Debian codenames we support.
SUPPORTED_CODENAMES=("sid" "testing" "bookworm" "bullseye")
# Ubuntu codenames we support (latest two LTS + latest non-LTS)
SUPPORTED_CODENAMES+=("jammy" "focal" "lunar")

# Use the default GPG key. Must be explicitly configured
code_signing_key_fingerprint="$(grep -m 1 "default-key" ~/.gnupg/gpg.conf | awk '{print $2}')"

function generate_repository_configuration {
    local codename=$1

    echo -e "Origin: repository.mullvad.net
Label: Mullvad apt repository
Description: Mullvad package repository for Debian/Ubuntu
Codename: $codename
Architectures: amd64 arm64
Components: main
SignWith: $code_signing_key_fingerprint"
}

function generate_deb_distributions_content {
    local distributions=""
    for codename in "${SUPPORTED_CODENAMES[@]}"; do
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

for deb_path in "$artifact_dir"/MullvadVPN-"$version"*.deb; do
    for codename in "${SUPPORTED_CODENAMES[@]}"; do
        add_deb_to_repo "$deb_path" "$codename"
        echo ""
    done
    echo ""
done
