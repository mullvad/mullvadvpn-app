#!/usr/bin/env bash

# This script prepares for a release. Run it with the release version as the first argument and it
# will update version numbers, commit and add a signed tag.

set -eu

if [[ "$#" != "1" ]]; then
    echo "Please give the release version as the first and only argument to this script."
    echo "For example: '2018.1-beta3' for a beta release, or '2018.6' for a stable one."
    exit 1
fi
VERSION=$1

if [[ $(echo $VERSION | egrep '^[0-9]{4}\.[0-9]+(-(beta|alpha)[0-9]+)?$') == "" ]]; then
    echo "Invalid version format. Please specify version as:"
    echo "<YEAR>.<NUMBER>[-(beta|alpha)<NUMBER>"
    exit 1
fi

if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
    echo "Dirty working directory! Will not accept that for an official release."
    exit 1
fi

if [[ $(grep $VERSION CHANGELOG.md) == "" ]]; then
    echo "It looks like you did not add $VERSION to the changelog?"
    echo "Please make sure the changelog is up to date and correct before you proceed."
    exit 1
fi

echo "Updating version in package.json..."
SEMVER_VERSION=`echo $VERSION | sed -re 's/($|-.*)/.0\1/g'`
sed -i -re "s/\"version\": \"[^\"]+\",/\"version\": \"$SEMVER_VERSION\",/g" package.json

echo "Commiting package.json change to git..."
git commit -S package.json -m "Updating version in package.json"

echo "Tagging current git commit with release tag $VERSION..."
git tag -s $VERSION -m $VERSION

echo "Done with preparations, going to compile and package stage..."
./build_release.sh

echo "DONE! Don't forget to push any commits and tags created by this release script!"