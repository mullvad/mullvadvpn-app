#!/usr/bin/env bash
#
# Buildserver configuration. Runtime values are defined here instead of
# the scripts where they are used.

# Servers to upload build artifacts to.
export PRODUCTION_UPLOAD_SERVERS=("cdn.mullvad.net")

# Where to publish new app artifact notification files to, so that
# build-linux-repositories picks it up.
# Keep in sync with build-linux-repositories-config.sh
#export LINUX_REPOSITORY_INBOX_DIR_BASE="PLEASE CONFIGURE ME"

# What container volumes cargo should put caches in.
# Specify differently if running multiple builds in parallel on one machine,
# so they don't use the same cache.
export CARGO_TARGET_VOLUME_NAME="cargo-target"
export CARGO_REGISTRY_VOLUME_NAME="cargo-registry"

# Where buildserver-build.sh should move artifacts (on Linux) and where
# buildserver-upload.sh should pick artifacts to upload
export UPLOAD_DIR="PLEASE CONFIGURE ME"
