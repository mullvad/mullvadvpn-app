#!/usr/bin/env bash

# Check the changelog and release notes to make sure they are correct.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."

VERSION_NAME=$1

if [[ -z ${VERSION_NAME+x} ]]; then
    echo "Please give the release version as an argument to this script."
    echo "For example: '2018.1-beta3' for a beta release, or '2018.6' for a stable one."
    exit 1
fi

echo "$VERSION_NAME"

if [[ $VERSION_NAME != *"alpha"* && $VERSION_NAME != *"-dev-"* &&
    $(grep "^## \\[android/$VERSION_NAME\\] - " android/CHANGELOG.md) == "" ]]; then

    echo "It looks like you did not add $VERSION_NAME to the changelog?"
    echo "Please make sure the changelog is up to date and correct before you proceed."
    exit 1
fi

release_notes=$(<"android/src/main/play/release-notes/en-US/default.txt")
if [[ $VERSION_NAME != *"alpha"* && $VERSION_NAME != *"-dev-"* && -z $release_notes ]]; then
    echo "The release notes file is empty!"
    echo "Beta and Stable require a release notes file."
    exit 1
fi

if [[ "${#release_notes}" -gt 500 ]]; then
    echo "The number of characters in the relase notes may not exceed 500"
    echo "Current number of charachers ${#release_notes}"
    exit 1
fi
