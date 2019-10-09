#!/usr/bin/env bash

# This script prepares for a release. Run it with the release version as the first argument and it
# will update version numbers, commit and add a signed tag.

set -eu

if [[ "$#" != "1" ]]; then
    echo "Please give the release version as the first and only argument to this script."
    echo "For example: '2018.1-beta3' for a beta release, or '2018.6' for a stable one."
    exit 1
fi
PRODUCT_VERSION=$1

if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
    echo "Dirty working directory! Will not accept that for an official release."
    exit 1
fi

if [[ $(grep $PRODUCT_VERSION CHANGELOG.md) == "" ]]; then
    echo "It looks like you did not add $PRODUCT_VERSION to the changelog?"
    echo "Please make sure the changelog is up to date and correct before you proceed."
    exit 1
fi

echo "Updating version in metadata files..."
./version_metadata.sh inject $PRODUCT_VERSION

echo "Syncing Cargo.lock with new version numbers"
source env.sh ""
cargo +stable --locked build

echo "Commiting metadata changes to git..."
git commit -S -m "Updating version in package files" \
    gui/package.json \
    gui/package-lock.json \
    mullvad-daemon/Cargo.toml \
    mullvad-cli/Cargo.toml \
    mullvad-problem-report/Cargo.toml \
    talpid-openvpn-plugin/Cargo.toml \
    Cargo.lock \
    android/build.gradle \
    dist-assets/windows/version.h

echo "Tagging current git commit with release tag $PRODUCT_VERSION..."
git tag -s $PRODUCT_VERSION -m $PRODUCT_VERSION

./version_metadata.sh delete-backup

echo "==================================================="
echo "DONE preparing for a release! Now do the following:"
echo " 1. Push the commit and tag created by this script"
echo "    after you have verified they are correct"
echo "     $ git push"
echo "     $ git push origin $PRODUCT_VERSION"
echo " 2. On each platform where you want to create a"
echo "    release artifact, check out the tag and build:"
echo "     $ git fetch"
echo "     $ git checkout $PRODUCT_VERSION"
echo "     $ ./build.sh"
echo "==================================================="
