#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

./gh-ready-check

version=$1
artifact_dir=$2

echo ">>> Downloading changelog"
changelog_path=$(mktemp)
curl -o "$changelog_path" --progress-bar \
  "https://raw.githubusercontent.com/mullvad/mullvadvpn-app/refs/tags/$version/CHANGELOG.md"

changelog_end_version_pattern="20[0-9]\{2\}\.[0-9]\{1,2\}"
if [[ $version == *-beta* ]]; then
    changelog_end_version_pattern=".*"
fi

changelog_extract=$(sed -n "/^## \[$version\]/,/^## \[$changelog_end_version_pattern\]/p" "$changelog_path")

changelog=$(echo "$changelog_extract" | sed '$d' | \
    awk 'NF { last = last ? last ORS $0 : $0 } END { print last }')

release_flags=(
  --draft
  --repo "git@github.com:mullvad/mullvadvpn-app"
  --verify-tag
  --notes-file -
  --title "$version"
)

previous_release=$(echo "$changelog_extract" | tail -1 | grep -oP '## \[\K[^\]]+')

body="This release is for desktop only."
if [[ $version == *-beta* ]]; then
    body+="\n\nHere is a list of all changes since last release [$previous_release](https://github.com/mullvad/mullvadvpn-app/releases/tag/$previous_release):"
    release_flags+=(--prerelease)
else
    body+="\n\nHere is a list of all changes since last stable release [$previous_release](https://github.com/mullvad/mullvadvpn-app/releases/tag/$previous_release):"
    release_flags+=(--latest)
fi

version_count=$(echo "$changelog" | grep -c "^## ")
if [ "$version_count" -eq 1 ]; then
    changelog=$(echo "$changelog" | tail -n +2)
fi

body+="\n$changelog"

echo ""
echo ">>> Creating GitHub release"
# shellcheck disable=SC2059
# shellcheck disable=SC2046
printf "$body" | gh release create "${release_flags[@]}" "$version" $(printf "%s " "$artifact_dir"/*)

echo ""
echo "The above URL contains the text \"untagged\", but don't worry it is tagged properly and everything will look correct once it's published."
