#!/usr/bin/env bash

set -eu
shopt -s nullglob globstar

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# shellcheck source=ci/buildserver-config.sh
source "$SCRIPT_DIR/buildserver-config.sh"

while true; do
    sleep 10

    for version_dir in "$RELEASE_INBOX_DIR"/*; do
        version=$(basename "$version_dir")
        pkg_name="MullvadVPN-$version"

        for ext in .exe _arm64.exe _x64.exe _amd64.deb _arm64.deb _x86_64.rpm _aarch64.rpm .pkg; do
            pkg_filename="$pkg_name$ext"
            if ! [ -f "$version_dir/$pkg_filename" ]; then
                continue 2
            fi

            "$RELEASE_SCRIPT_DIR/verify-artifacts.sh" "$pkg_filename"
        done

        ./release-scripts/3-verify-build "$version" --wait
        ./release-scripts/publish-github-release.sh "$version" "$version_dir"

        rm -f "$version_dir/$pkg_name"*
    done
done
