#!/usr/bin/env bash

# This script should be executed as root.
#
# Run `openwrt/build.sh` before running this script.
#
# The architecture is assumed to be x86/x86_64 musl for now.
#
# Example: bash build.sh --release && bash package.sh 2025.13

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_REPO_ROOT="$SCRIPT_DIR/.."

if [ -z "$1" ]; then
    echo "No version was supplied"
    echo "Usage: $0 <version>"
    exit 1
fi

PACKAGE_NAME="mullvad_$1.x86_64.ipk"
# Temporary mirror of .ipk archive hierarchy.
IPK_FILES=`mktemp -d`
# The intermediare archive that will eventually be bundled into the final .ipk.
WORK_DIR=`mktemp -d`

echo "Assembling $PACKAGE_NAME"

# First copy over the .ipk hierarchy to the temporary folder.
cp -r ./mullvad/* "$IPK_FILES/"

# Then, move app artifacts to the appropriate location in the .ipk archive hierarchy.
# This will be reflected in the OpenWRT host.
cp "$APP_REPO_ROOT"/target/x86_64-unknown-linux-musl/release/{mullvad,mullvad-daemon} "$IPK_FILES/data/usr/bin/"

# Then make sure everything is owned by root before creating the .ipk archive.
chown -R root:root "$IPK_FILES/"
# TODO: Ensure that all files that ought to be executable are executable.

# Move all artifacts to temporary directory.
pushd "$IPK_FILES/control"
tar -czf "$WORK_DIR/control.tar.gz" ./*
popd

pushd "$IPK_FILES/data"
tar -czf "$WORK_DIR/data.tar.gz" ./*
popd

cp "$IPK_FILES/debian-binary" "$WORK_DIR"

# Assemble .ipk
pushd "$WORK_DIR"
tar -czf "$PACKAGE_NAME" ./*
popd

# This artifact may be copied over to an OpenWRT router.
mv "$WORK_DIR/$PACKAGE_NAME" ./

rm -rf "$WORK_DIR"
rm -rf "$IPK_FILES"

echo "Finished building ipk. Enjoy!"
