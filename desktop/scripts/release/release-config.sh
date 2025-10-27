#!/usr/bin/env bash

# Configuration variables shared between the release scripts in this directory.

# Where the release scripts and programs store temporary data
export DATA_DIR="$HOME/.local/share/mullvad-release"

# Where to download app installers locally during the release process.
# This value is also hardcoded into the `mullvad-release` binary and
# has to be in sync with that value
export ARTIFACT_DIR="$DATA_DIR/artifacts"

# Mullvad code signing key and fingerprint
export MULLVAD_CODE_SIGNING_KEY_PATH="../../../ci/keys/1.mullvad_signing.pub"
export MULLVAD_CODE_SIGNING_KEY_FINGERPRINT="A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF"
