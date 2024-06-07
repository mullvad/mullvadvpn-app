#!/usr/bin/env bash
#
# Buildserver configuration. Runtime values are defined here instead of
# the scripts where they are used.

# Servers to upload build artifacts to.
export PRODUCTION_UPLOAD_SERVERS=("cdn.mullvad.net")

# Base directory `publish-linux-repositories.sh` will store artifacts under for
# `build-linux-repositories.sh` to pick them up from.
export LINUX_REPOSITORY_TMP_STORE_DIR_BASE="/tmp/app-artifacts/"

# Temporary storage of .src files before atomically moving into
# $LINUX_REPOSITORY_INBOX_DIR_BASE/$environment/$stable_or_beta
export LINUX_REPOSITORY_NOTIFY_FILE_TMP_DIR="/tmp/linux-repositories"
# Where to publish new app artifact notification files to, so that
# build-linux-repositories picks it up.
# Keep in sync with build-linux-repositories-config.sh
export LINUX_REPOSITORY_INBOX_DIR_BASE="/tmp/linux-repositories"

# What container volumes cargo should put caches in.
# Specify differently if running multiple builds in parallel on one machine,
# so they don't use the same cache.
export CARGO_TARGET_VOLUME_NAME="cargo-target"
export CARGO_REGISTRY_VOLUME_NAME="cargo-registry"

# Where buildserver-build.sh should move artifacts (on Linux) and where
# buildserver-upload.sh should pick artifacts to upload
export UPLOAD_DIR="PLEASE CONFIGURE ME"
