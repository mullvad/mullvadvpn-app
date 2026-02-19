#!/usr/bin/env bash

# Configuration variables shared between the release scripts in this directory.

# Where the release scripts and programs store temporary data
export DATA_DIR="$HOME/.local/share/mullvad-release-android"

export WORK_DIR="$DATA_DIR/work/"
export PUBLISHED_DIR="$DATA_DIR/currently_published/"

# Mullvad code signing key and fingerprint
export MULLVAD_CODE_SIGNING_KEY_PATH="../../../ci/keys/1.mullvad_signing.pub"
export MULLVAD_CODE_SIGNING_KEY_FINGERPRINT="A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF"

# The user on the buildserver that builds and uploads artifacts to the cdn servers
export BUILDSERVER_BUILDUSER="build"
