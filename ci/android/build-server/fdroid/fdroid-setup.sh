#!/usr/bin/env bash

# Sets up the fdroid repo directory with the required files from this folder.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

FDROID_ROOT_DIR=${1:?'Usage: fdroid-setup.sh <fdroid-repo-dir>'}

if [[ ! -d "$FDROID_ROOT_DIR" ]]; then
    echo "Error: not a directory: $FDROID_ROOT_DIR"
    exit 1
fi

FDROID_ROOT_DIR="$(cd "$FDROID_ROOT_DIR" && pwd)"

# Copy config file
cp "config.yml" "$FDROID_ROOT_DIR"

# Copy the metadata file
mkdir -p "$FDROID_ROOT_DIR/metadata"
cp "net.mullvad.mullvadvpn.yml" "$FDROID_ROOT_DIR/metadata"

# Copy the app page icon file
mkdir -p "$FDROID_ROOT_DIR/metadata/net.mullvad.mullvadvpn/en-US/images"
cp "icon.png" "$FDROID_ROOT_DIR/metadata/net.mullvad.mullvadvpn/en-US/images"

# Create the repo folder
mkdir -p "$FDROID_ROOT_DIR/repo"

# Create the changelogs folder
mkdir -p "$FDROID_ROOT_DIR/metadata/net.mullvad.mullvadvpn/en-US/changelogs"
