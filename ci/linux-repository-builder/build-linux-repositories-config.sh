#!/usr/bin/env bash

# The directory to use as inbox. This is where .src files are read
#export LINUX_REPOSITORY_INBOX_DIR_BASE="PLEASE CONFIGURE ME"

# GPG key to sign the repositories with
export CODE_SIGNING_KEY_FINGERPRINT="A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF"

# Debian codenames we support.
SUPPORTED_DEB_CODENAMES=("sid" "testing" "trixie" "bookworm" "bullseye")
# Ubuntu codenames we support. Latest two LTS. But when adding a new
# don't immediately remove the oldest one. Allow for some transition period
# with the last three.
SUPPORTED_DEB_CODENAMES+=("noble" "jammy" "focal")
# ... + latest non-LTS Ubuntu. We officially only support the latest non-LTS.
# But to not cause too much disturbance just when Ubuntu is released, we keep
# the last two codenames working in the repository.
SUPPORTED_DEB_CODENAMES+=("plucky" "oracular")
export SUPPORTED_DEB_CODENAMES

export SUPPORTED_RPM_ARCHITECTURES=("x86_64" "aarch64")

export REPOSITORIES=("stable" "beta")

export PRODUCTION_LINUX_REPOSITORY_PUBLIC_URL="https://repository.mullvad.net"
export STAGING_LINUX_REPOSITORY_PUBLIC_URL="https://repository.stagemole.eu"
export DEV_LINUX_REPOSITORY_PUBLIC_URL="https://repository.devmole.eu"

# Servers to upload Linux deb/rpm repositories to
export PRODUCTION_REPOSITORY_SERVER="cdn.mullvad.net"
export STAGING_REPOSITORY_SERVER="cdn.stagemole.eu"
export DEV_REPOSITORY_SERVER="cdn.devmole.eu"

#export PRODUCTION_BUNNYCDN_PULL_ZONE_ID="PLEASE CONFIGURE ME"
#export STAGING_BUNNYCDN_PULL_ZONE_ID="PLEASE CONFIGURE ME"
#export BUNNYCDN_API_KEY="PLEASE CONFIGURE ME"
