#!/usr/bin/env bash

set -eu

CODE_SIGNING_KEY_FINGERPRINT=${CODE_SIGNING_KEY_FINGERPRINT:-"A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF"}

BASEDIR="deb"

# Debian codenames we support.
SUPPORTED_CODENAMES=("sid" "testing" "bookworm" "bullseye")
# Ubuntu codenames we support (latest two LTS + latest non-LTS)
SUPPORTED_CODENAMES+=("jammy" "focal" "lunar")

function generate_repository_configuration {
    local codename=$1
    echo -e "Origin: repository.devmole.eu
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
    reprepro -V --basedir "$BASEDIR" --component main includedeb "$codename" "$deb_path"
}

echo "Generating deb repository into $BASEDIR/"
mkdir -p "$BASEDIR/conf"

echo "Writing repository configuration to $BASEDIR/conf/distributions"
generate_deb_distributions_content > "$BASEDIR/conf/distributions"
echo ""

for deb_path in dist/MullvadVPN-*.deb; do
    for codename in "${SUPPORTED_CODENAMES[@]}"; do
        # Add all releases, beta and stable, to the -testing repository
        add_deb_to_repo "$deb_path" "$codename"-testing

        # If this is a stable release, also add it to the stable repo
        if [[ $(basename "$deb_path") != *"-beta"* ]]; then
            add_deb_to_repo "$deb_path" "$codename"
        fi
        echo ""
    done
    echo ""
done
