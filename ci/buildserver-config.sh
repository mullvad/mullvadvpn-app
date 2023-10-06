#!/usr/bin/env bash
#
# Buildserver configuration. Runtime values are defined here instead of
# the scripts where they are used.

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Which gpg key to sign things with
export CODE_SIGNING_KEY_FINGERPRINT="A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF"

# Debian codenames we support.
SUPPORTED_DEB_CODENAMES=("sid" "testing" "bookworm" "bullseye")
# Ubuntu codenames we support (latest two LTS + latest non-LTS)
SUPPORTED_DEB_CODENAMES+=("jammy" "focal" "lunar")
export SUPPORTED_DEB_CODENAMES

# Where to store deb repositories locally when building releases
export DEB_REPOSITORY_ARCHIVE_DIR="$SCRIPT_DIR/deb"

# Servers to upload Linux deb/rpm repositories to
export STAGING_LINUX_REPOSITORY_SERVERS=("se-got-cdn-001.stagemole.eu" "se-got-cdn-002.stagemole.eu")
export PRODUCTION_LINUX_REPOSITORY_SERVERS=("se-got-cdn-111.mullvad.net" "se-mma-cdn-101.mullvad.net")
