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

# Regex that only matches valid Mullvad VPN versions. It also captures
# relevant values into capture groups, read out via BASH_REMATCH[x]
VERSION_REGEX="^20([0-9]{2})\.([1-9][0-9]?)(-beta([1-9][0-9]?))?$"
if [[ ! $VERSION =~ $VERSION_REGEX ]]; then
    echo "Invalid version format. Please specify version as:"
    echo "<YEAR>.<NUMBER>[-beta<NUMBER>]"
    exit 1
fi
VERSION_YEAR=$(printf "%02d" ${BASH_REMATCH[1]})
VERSION_NUMBER=$(printf "%02d" ${BASH_REMATCH[2]})
VERSION_PATCH="00"
VERSION_BETA=$(printf "%02d" ${BASH_REMATCH[4]:-99})
ANDROID_VERSION_CODE=${VERSION_YEAR}${VERSION_NUMBER}${VERSION_PATCH}${VERSION_BETA}

SEMVER_VERSION=$(echo $VERSION | sed -Ee 's/($|-.*)/.0\1/g')

if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
    echo "Dirty working directory! Will not accept that for an official release."
    exit 1
fi

if [[ $(grep $VERSION CHANGELOG.md) == "" ]]; then
    echo "It looks like you did not add $VERSION to the changelog?"
    echo "Please make sure the changelog is up to date and correct before you proceed."
    exit 1
fi

echo "Updating version in metadata files..."
sed -i.bak -Ee "s/\"version\": \"[^\"]+\",/\"version\": \"$SEMVER_VERSION\",/g" \
    gui/package.json
sed -i.bak -Ee "s/^version = \"[^\"]+\"\$/version = \"$SEMVER_VERSION\"/g" \
    mullvad-daemon/Cargo.toml \
    mullvad-cli/Cargo.toml \
    mullvad-problem-report/Cargo.toml

sed -i.bak -Ee "s/versionCode [0-9]+/versionCode $ANDROID_VERSION_CODE/g" \
    android/build.gradle
sed -i.bak -Ee "s/versionName \"[^\"]+\"/versionName \"$VERSION\"/g" \
    android/build.gradle


echo "Syncing Cargo.lock with new version numbers"
source env.sh ""
cargo build

(cd gui/ && npm install) || exit 1

echo "Commiting metadata changes to git..."
git commit -S -m "Updating version in package files" \
    gui/package.json \
    gui/package-lock.json \
    mullvad-daemon/Cargo.toml \
    mullvad-cli/Cargo.toml \
    mullvad-problem-report/Cargo.toml \
    Cargo.lock

echo "Tagging current git commit with release tag $VERSION..."
git tag -s $VERSION -m $VERSION


echo "==================================================="
echo "DONE preparing for a release! Now do the following:"
echo " 1. Push the commit and tag created by this script"
echo "    after you have verified they are correct"
echo "     $ git push"
echo "     $ git push origin $VERSION"
echo " 2. On each platform where you want to create a"
echo "    release artifact, check out the tag and build:"
echo "     $ git fetch"
echo "     $ git checkout $VERSION"
echo "     $ ./build.sh"
echo "==================================================="
