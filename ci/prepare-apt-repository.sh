#!/usr/bin/env bash

set -eu

CODE_SIGNING_KEY_FINGERPRINT=${CODE_SIGNING_KEY_FINGERPRINT:-"A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF"}

artifact_dir=$1
version=$2
basedir=$3

# Slimmed down set for testing
SUPPORTED_CODENAMES=("bookworm")
SUPPORTED_CODENAMES+=("jammy")

# # Debian codenames we support.
# SUPPORTED_CODENAMES=("sid" "testing" "bookworm" "bullseye")
# # Ubuntu codenames we support (latest two LTS + latest non-LTS)
# SUPPORTED_CODENAMES+=("jammy" "focal" "lunar")

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
    reprepro -V --basedir "$basedir" --component main includedeb "$codename" "$deb_path"
}

echo "Generating deb repository into $basedir/"
mkdir -p "$basedir/conf"

echo "Writing repository configuration to $basedir/conf/distributions"
generate_deb_distributions_content > "$basedir/conf/distributions"
echo ""

for deb_path in "$artifact_dir"/MullvadVPN-"$version"*.deb; do
    for codename in "${SUPPORTED_CODENAMES[@]}"; do
        add_deb_to_repo "$deb_path" "$codename"
        echo ""
    done
    echo ""
done
