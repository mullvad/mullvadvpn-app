#!/usr/bin/env bash
#
# Usage: ./regenerate-apt-repositories.sh <app version> <repository dir>
#
# Will regenerate deb repositories in <repository dir> with the content of already existing
# repositories for <version>.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

version=$1
repo_dir=$2

temp_dir=$(mktemp -d)
cp "$repo_dir/$version/pool/main/m/mullvad-vpn/*" "$temp_dir"

"$SCRIPT_DIR/prepare-apt-repository.sh" "$temp_dir" "$version" "$repo_dir"

rm -rf "$temp_dir"
