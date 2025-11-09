#!/usr/bin/env bash

# This script should be executed as root.
#
# Run `openwrt/build-{x86,armv7}.sh` before running this script.
#
# Arguments:
#   version: App version number
#   CPU architecture: Target CPU architecture. Either x86 or armv7.
#   --minify: Use `upx` to compress binaries.
#
# Example: bash build-x86.sh --release   && bash package.sh 2025.13 x86
#          bash build-armv7.sh --release && bash package.sh 2025.13 armv7
#
# Depends: upx (optional)

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_REPO_ROOT="$SCRIPT_DIR/.."

if [ -z "$1" ]; then
    echo "No version was supplied"
    echo "Usage: $0 <version> <x86 | armv7>"
    exit 1
fi

case $2 in
    x86)
        PACKAGE_NAME="mullvad_$1.x86_64.ipk"
        IPK_SCAFFOLDING="mullvad-x86"
        CARGO_TARGET_DIR="$APP_REPO_ROOT"/target/x86_64-unknown-linux-musl
        ;;
    armv7)
        PACKAGE_NAME="mullvad_$1.armv7.ipk"
        IPK_SCAFFOLDING="mullvad-armv7"
        CARGO_TARGET_DIR="$APP_REPO_ROOT"/target/armv7-unknown-linux-musleabihf
        ;;
    *)
        echo "No architecture was supplied"
        echo "Usage: $0 <version> <x86 | armv7>"
        exit 1
        ;;
esac

# Temporary mirror of .ipk archive hierarchy.
IPK_FILES=`mktemp -d`
# The intermediare archive that will eventually be bundled into the final .ipk.
WORK_DIR=`mktemp -d`

echo "Assembling $PACKAGE_NAME"

# First copy over the .ipk hierarchy to the temporary folder.
cp -r "$IPK_SCAFFOLDING"/* "$IPK_FILES/"

# Then, move app artifacts to the appropriate location in the .ipk archive hierarchy.
# This will be reflected in the OpenWRT host.
mkdir -p "$IPK_FILES/data/usr/bin/" # Ensure that the /data/usr/bin folder exists before copying files.
cp "$CARGO_TARGET_DIR"/release/{mullvad,mullvad-daemon} "$IPK_FILES/data/usr/bin/"

# Optional: If `--minify` is used, invoke `upx` to compress binaries before packaging.
case $3 in
    --minify) upx --best --lzma "$IPK_FILES"/data/usr/bin/* || true ;; # Disregard failure, minify on a best-effort basis.
    *) ;;
esac

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
