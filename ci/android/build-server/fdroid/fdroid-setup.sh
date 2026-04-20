#!/usr/bin/env bash

# Sets up the fdroid repo directory with the required files from this folder.
# The set up is the following
# Adds the following files in the folders:
# FDROID_REPO_DIR/config.yml
# FDROID_REPO_DIR/metadata/net.mullvad.mullvadvpn.yml
# FDROID_REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/images/icon.png
# Creates the following empty folders:
# FDROID_REPOR_DIR/repo
# FDROID_REPO_DIR/metadata/net.mullvad.mullvadvpn/en-US/changelogs

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

FDROID_REPO_DIR=${1:?'Usage: fdroid-setup.sh <fdroid-repo-dir>'}

if [[ ! -d "$FDROID_REPO_DIR" ]]; then
    echo "Error: not a directory: $FDROID_REPO_DIR"
    exit 1
fi

cd "$FDROID_REPO_DIR"

# Copy config file
cp "$SCRIPT_DIR/config.yml" .

# Copy the metadata file
mkdir -p "metadata"
cp "$SCRIPT_DIR/net.mullvad.mullvadvpn.yml" "metadata"

# Copy the app page icon file
mkdir -p "metadata/net.mullvad.mullvadvpn/en-US/images"
cp "$SCRIPT_DIR/icon.png" "metadata/net.mullvad.mullvadvpn/en-US/images"

# Create the repo folder
mkdir -p "repo"

# Create the changelogs folder
mkdir -p "metadata/net.mullvad.mullvadvpn/en-US/changelogs"
