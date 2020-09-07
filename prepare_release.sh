#!/usr/bin/env bash

# This script prepares for a release. Run it with the release version as the first argument and it
# will update version numbers, commit and add a signed tag.

set -eu

PRODUCT_VERSION=""
ANDROID="false"
DESKTOP="false"

for argument in "$@"; do
    case "$argument" in
        "--android")
            ANDROID="true"
            ;;
        "--desktop")
            DESKTOP="true"
            ;;
        -*)
            echo "Unknown option \"$argument\""
            exit 1
            ;;
        *)
            PRODUCT_VERSION="$argument"
            ;;
    esac
done

if [ -z "$PRODUCT_VERSION" ]; then
    echo "Please give the release version as an argument to this script."
    echo "For example: '2018.1-beta3' for a beta release, or '2018.6' for a stable one."
    exit 1
fi

if [[ "$ANDROID" != "true" && "$DESKTOP" != "true" ]]; then
    echo "Please specify if the release is for the desktop app and/or for Android app."
    echo "For example: --android --desktop"
    exit 1
fi

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
./version-metadata.sh inject $PRODUCT_VERSION

echo "Syncing Cargo.lock with new version numbers"
source env.sh
# If cargo exits with a non zero exit status and it's not a timeout (exit code 124) it's an error
set +e
timeout 5s cargo +stable build
if [[ $? != 0 && $? != 124 ]]; then
    exit 1
fi
set -e

echo "Commiting metadata changes to git..."
git commit -S -m "Updating version in package files" \
    gui/package.json \
    gui/package-lock.json \
    mullvad-daemon/Cargo.toml \
    mullvad-cli/Cargo.toml \
    mullvad-problem-report/Cargo.toml \
    mullvad-setup/Cargo.toml \
    mullvad-exclude/Cargo.toml \
    talpid-openvpn-plugin/Cargo.toml \
    Cargo.lock \
    android/build.gradle \
    dist-assets/windows/version.h

echo "Tagging current git commit with release tag $PRODUCT_VERSION..."

if [[ "$ANDROID" == "true" ]]; then
    git tag -s "android/$PRODUCT_VERSION" -m "android/$PRODUCT_VERSION"
fi
if [[ "$DESKTOP" == "true" ]]; then
    git tag -s $PRODUCT_VERSION -m $PRODUCT_VERSION
fi

./version-metadata.sh delete-backup

echo "================================================="
echo "| DONE preparing for a release!                 |"
echo "|    Now push the tag created by this script    |"
echo "|    after you have verified it is correct:     |"
echo "|        $ git push origin $PRODUCT_VERSION     |"
echo "================================================="
