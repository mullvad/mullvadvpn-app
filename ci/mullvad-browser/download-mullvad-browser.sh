#!/usr/bin/bash -e

set -eu

BROWSER_RELEASES=("stable" "alpha")
REPOSITORIES=("stable" "beta")

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TMP_DIR=$(mktemp -qdt mullvad-browser-tmp-XXXXXXX)
WORKDIR="$SCRIPT_DIR/mullvad-browser-download"
NOTIFY_DIR=/tmp/linux-repositories/production


function usage() {
    echo "Usage: $0"
    echo
    echo "This script downloads, verifies, and notifies about Mullvad browser packages."
    echo
    echo "Options:"
    echo "  -h | --help    Show this help message and exit."
    exit 1
}


function main() {
    local package_filename_base=$1
    local extension=$2
    PACKAGE_FILENAME="$package_filename_base.$extension"

    PACKAGE_URL=https://cdn.mullvad.net/browser/$PACKAGE_FILENAME
    SIGNATURE_URL=$PACKAGE_URL.asc

    echo "[#] Downloading $PACKAGE_FILENAME"
    if ! wget --quiet "$PACKAGE_URL"; then
        echo "[!] Failed to download $PACKAGE_URL"
        exit 1
    fi

    echo "[#] Downloading $PACKAGE_FILENAME.asc"
    if ! wget --quiet "$SIGNATURE_URL"; then
        echo "[!] Failed to download $SIGNATURE_URL"
        exit 1
    fi

    echo "[#] Verifying $PACKAGE_FILENAME signature"
    if ! gpg --verify "$PACKAGE_FILENAME".asc "$PACKAGE_FILENAME"; then
        echo "[!] Failed to verify signature"
        rm "$PACKAGE_FILENAME" "$PACKAGE_FILENAME.asc"
        exit 1
    fi
    rm "$PACKAGE_FILENAME.asc"

    # Hack to get the architecture into the filename
    local filename_with_arch="${package_filename_base}_x86_64.$extension"
    mv "$PACKAGE_FILENAME" "$filename_with_arch"
    PACKAGE_FILENAME="$filename_with_arch"

    # Check if the deb package has changed since last time
    # Handle the bootstrap problem by checking if the "output file" even exists and just moving on if it doesn't
    if [[ -f "$WORKDIR/$PACKAGE_FILENAME" ]] && cmp "$PACKAGE_FILENAME" "$WORKDIR/$PACKAGE_FILENAME"; then
        echo "[#] $PACKAGE_FILENAME has not changed"
        rm "$PACKAGE_FILENAME"
        return
    fi

    echo "[#] $PACKAGE_FILENAME has changed"
    cp "$PACKAGE_FILENAME" "$WORKDIR/"
    # Leaving a file in `$TMP_DIR` is used as an indicator further down that something changed
}

if [[ ${1:-} == "-h" ]] || [[ ${1:-} == "--help" ]]; then
    usage
fi


if ! [[ -d $NOTIFY_DIR ]]; then
    echo "[!] $NOTIFY_DIR does not exist"
    exit 1
fi


if ! [[ -d $WORKDIR ]]; then
    echo "[#] Creating $WORKDIR"
    mkdir -p "$WORKDIR"
fi


pushd "$TMP_DIR" > /dev/null
function delete_tmp_dir {
    echo "[#] Exiting and deleting $TMP_DIR"
    rm -rf "$TMP_DIR"
}
trap 'delete_tmp_dir' EXIT


echo "[#] Configured releases are: ${BROWSER_RELEASES[*]}"
for release in "${BROWSER_RELEASES[@]}"; do
    main "mullvad-browser-$release" "deb"
    main "mullvad-browser-$release" "rpm"
done

if [[ -z "$(ls -A "$TMP_DIR")" ]]; then
    echo "[#] No new browser build(s) exist"
    exit
fi

echo "[#] New browser build(s) exist"
for repository in "${REPOSITORIES[@]}"; do
    inbox_dir="$NOTIFY_DIR/$repository"

    REPOSITORY_TMP_ARTIFACT_DIR=$(mktemp -qdt mullvad-browser-tmp-XXXXXXX)
    cp "$WORKDIR"/* "$REPOSITORY_TMP_ARTIFACT_DIR"

    repository_notify_file="$inbox_dir/browser.src"
    echo "[#] Notifying $repository_notify_file"
    echo "$REPOSITORY_TMP_ARTIFACT_DIR" > "$repository_notify_file"
done
