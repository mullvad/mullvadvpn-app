#!/usr/bin/env bash

# The directory to use as inbox. This is where .src files are read
#export LINUX_REPOSITORY_INBOX_DIR_BASE="PLEASE CONFIGURE ME"

# GPG key to sign the repositories with
export CODE_SIGNING_KEY_FINGERPRINT="A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF"

# Debian codenames we support.
# On top of these we also add the name of the repository as a codename as
# well. Meaning the `stable` repository will also have a `stable` codename,
# and `beta` will have `beta` as a codename. We are transitioning to users
# using these fixed codenames, and away from distro specific code names.
# SO DO NOT ADD NEW CODENAMES HERE.
SUPPORTED_DEB_CODENAMES=("sid" "testing")
SUPPORTED_DEB_CODENAMES+=("trixie") # 13
SUPPORTED_DEB_CODENAMES+=("bookworm") # 12
SUPPORTED_DEB_CODENAMES+=("bullseye") # 11
# Ubuntu codenames we support. Same as above: DO NOT ADD NEW CODENAMES HERE.
SUPPORTED_DEB_CODENAMES+=("plucky") # 25.04
SUPPORTED_DEB_CODENAMES+=("noble") # 24.04 LTS
SUPPORTED_DEB_CODENAMES+=("jammy") # 22.04 LTS
SUPPORTED_DEB_CODENAMES+=("focal") # 20.04 LTS
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
