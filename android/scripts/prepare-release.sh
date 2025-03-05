#!/usr/bin/env bash

# This script prepares for a release. Run it with the release version as the first argument and it
# will update version numbers, commit and add a signed tag.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."

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

if [[ $PRODUCT_VERSION != *"alpha"* &&
    $(grep "^## \\[android/$PRODUCT_VERSION\\] - " android/CHANGELOG.md) == "" ]]; then

    echo "It looks like you did not add $PRODUCT_VERSION to the changelog?"
    echo "Please make sure the changelog is up to date and correct before you proceed."
    exit 1
fi

echo "Generate relays.json"
mkdir -p dist-assets/relays
cargo run -q -p mullvad-api --bin relay_list > dist-assets/relays/relays.json

git add dist-assets/relays/relays.json
git commit -S -m "Add relay list to bundle with $PRODUCT_VERSION"

echo "$PRODUCT_VERSION" > dist-assets/android-version-name.txt
ANDROID_VERSION="$PRODUCT_VERSION" cargo run -q --bin mullvad-version versionCode > \
    dist-assets/android-version-code.txt
git commit -S -m "Update android app version to $PRODUCT_VERSION" \
    dist-assets/android-version-name.txt \
    dist-assets/android-version-code.txt


echo "Tagging current git commit with release tag android/$PRODUCT_VERSION..."
git tag -s "android/$PRODUCT_VERSION" -m "android/$PRODUCT_VERSION"

echo "================================================="
echo "| DONE preparing for a release!                 |"
echo "|    Now push the tag created by this script    |"
echo "|    after you have verified it is correct:     |"
echo "|        $ git push origin $PRODUCT_VERSION"
echo "================================================="
