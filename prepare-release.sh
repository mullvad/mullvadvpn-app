#!/usr/bin/env bash

# This script prepares for a release. Run it with the release version as the first argument and it
# will update version numbers, commit and add a signed tag.

set -eu

ANDROID="false"
DESKTOP="false"
VERSION_METADATA_ARGS=""

for argument in "$@"; do
    case "$argument" in
        "--android")
            ANDROID="true"
            ;;
        "--desktop")
            DESKTOP="true"
            VERSION_METADATA_ARGS+="--desktop "
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

if [[ -z ${PRODUCT_VERSION+x} ]]; then
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

if [[ $DESKTOP == "true" && $(grep "CHANGE THIS BEFORE A RELEASE" gui/changes.txt) != "" ]]; then
    echo "It looks like you did not update gui/changes.txt"
    exit 1
fi

if [[ "$DESKTOP" == "true" && $(grep "^## \\[$PRODUCT_VERSION\\] - " CHANGELOG.md) == "" ]]; then
    echo "It looks like you did not add $PRODUCT_VERSION to the changelog?"
    echo "Please make sure the changelog is up to date and correct before you proceed."
    exit 1
fi

if [[ "$ANDROID" == "true" && $(grep "^## \\[android/$PRODUCT_VERSION\\] - " CHANGELOG.md) == "" ]]; then
    echo "It looks like you did not add $PRODUCT_VERSION to the changelog?"
    echo "Please make sure the changelog is up to date and correct before you proceed."
    exit 1
fi

if [[ "$DESKTOP" == "true" ]]; then
    echo "$PRODUCT_VERSION" > dist-assets/desktop-product-version.txt
    git commit -S -m "Update desktop app version to $PRODUCT_VERSION" \
        dist-assets/desktop-product-version.txt
fi

if [[ "$ANDROID" == "true" ]]; then
    echo "$PRODUCT_VERSION" > dist-assets/android-product-version.txt
    git commit -S -m "Update android app version to $PRODUCT_VERSION" \
        dist-assets/android-product-version.txt
fi

NEW_TAGS=""

if [[ "$ANDROID" == "true" ]]; then
    echo "Tagging current git commit with release tag android/$PRODUCT_VERSION..."

    git tag -s "android/$PRODUCT_VERSION" -m "android/$PRODUCT_VERSION"
    NEW_TAGS+=" android/$PRODUCT_VERSION"
fi
if [[ "$DESKTOP" == "true" ]]; then
    echo "Tagging current git commit with release tag $PRODUCT_VERSION..."

    git tag -s $PRODUCT_VERSION -m $PRODUCT_VERSION
    NEW_TAGS+=" $PRODUCT_VERSION"
fi

echo "================================================="
echo "| DONE preparing for a release!                 |"
echo "|    Now push the tag created by this script    |"
echo "|    after you have verified it is correct:     |"
echo "|        $ git push origin$NEW_TAGS"
echo "================================================="
