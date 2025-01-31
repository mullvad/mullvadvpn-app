#!/usr/bin/env bash

# This script prepares for a release. Run it with the release version as the first argument and it
# will update version numbers, commit and add a signed tag.

set -eu

for argument in "$@"; do
    case "$argument" in
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

if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
    echo "Dirty working directory! Will not accept that for an official release."
    exit 1
fi

desktop_changes_path=desktop/packages/mullvad-vpn/changes.txt
if [[ $(grep "CHANGE THIS BEFORE A RELEASE" $desktop_changes_path) != "" ]]; then
    echo "It looks like you did not update $desktop_changes_path"
    exit 1
fi

if [[ $(grep "^## \\[$PRODUCT_VERSION\\] - " CHANGELOG.md) == "" ]]; then
    echo "It looks like you did not add $PRODUCT_VERSION to the changelog?"
    echo "Please make sure the changelog is up to date and correct before you proceed."
    exit 1
fi

echo "$PRODUCT_VERSION" > dist-assets/desktop-product-version.txt
git commit -S -m "Update desktop app version to $PRODUCT_VERSION" \
    dist-assets/desktop-product-version.txt

echo "Tagging current git commit with release tag $PRODUCT_VERSION..."
git tag -s "$PRODUCT_VERSION" -m "$PRODUCT_VERSION"

echo "================================================="
echo "| DONE preparing for a release!                 |"
echo "|    Now push the tag created by this script    |"
echo "|    after you have verified it is correct:     |"
echo "|        $ git push origin $PRODUCT_VERSION"
echo "================================================="
