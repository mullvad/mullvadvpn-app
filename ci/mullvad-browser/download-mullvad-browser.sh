#!/usr/bin/bash -e

set -eu

# TODO: Uncomment when alpha is to be released
# BROWSER_RELEASES=("alpha" "stable")
BROWSER_RELEASES=("stable")
REPOSITORIES=("stable" "beta")
TMP_DIR=$(mktemp -qdt mullvad-browser-tmp-XXXXXXX)
WORKDIR=/tmp/mullvad-browser-download
NOTIFY_DIR=/tmp/linux-repositories/production


function usage() {
    echo "Usage: $0"
    echo
    echo "This script downloads, verifies, and notifies about Mullvad browser packages."
    echo
    echo "Options:"
    echo "  -h | --help	Show this help message and exit."
    exit 1
}


function main() {
    # mullvad-browser-stable.deb
    PACKAGE_FILENAME=$1
    PACKAGE_URL=https://cdn.mullvad.net/browser/$PACKAGE_FILENAME
    SIGNATURE_URL=$PACKAGE_URL.asc

    echo "[#] Downloading $PACKAGE_FILENAME"
    if ! wget --quiet --show-progress "$PACKAGE_URL"; then
        echo "[!] Failed to download $PACKAGE_URL"
        exit 1
    fi

    echo "[#] Downloading $PACKAGE_FILENAME.asc"
    if ! wget --quiet --show-progress "$SIGNATURE_URL"; then
        echo "[!] Failed to download $SIGNATURE_URL"
        exit 1
    fi

    echo "[#] Verifying $PACKAGE_FILENAME signature"
    if ! gpg --verify "$PACKAGE_FILENAME".asc; then
        echo "[!] Failed to verify signature"
        exit 1
    fi
    rm "$PACKAGE_FILENAME.asc"

    # Check if the deb package has changed since last time
    # Handle the bootstrap problem by checking if the "output file" even exists and just moving on if it doesn't
    if [[ -f "$WORKDIR/$PACKAGE_FILENAME" ]] && cmp "$PACKAGE_FILENAME" "$WORKDIR/$PACKAGE_FILENAME"; then
        echo "[#] $PACKAGE_FILENAME has not changed"
        rm "$PACKAGE_FILENAME"
        return
    fi

    echo "[#] $PACKAGE_FILENAME has changed"
    ln "$PACKAGE_FILENAME" $WORKDIR/
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
    mkdir -p $WORKDIR
fi


pushd "$TMP_DIR" > /dev/null


echo "[#] Configured releases are: ${BROWSER_RELEASES[*]}"
for release in "${BROWSER_RELEASES[@]}"; do
    main "mullvad-browser-$release.deb"
    main "mullvad-browser-$release.rpm"
done

if [[ -z "$(ls -A "$TMP_DIR")" ]]; then
    echo "[#] No new browser build(s) exist"
    exit
fi

echo "[#] New browser build(s) exist"
for repository in "${REPOSITORIES[@]}"; do
    inbox_dir="$NOTIFY_DIR/$repository"

    REPOSITORY_TMP_ARTIFACT_DIR=$(mktemp -qdt mullvad-browser-tmp-XXXXXXX)
    cp "$TMP_DIR"/* "$REPOSITORY_TMP_ARTIFACT_DIR"

    repository_notify_file="$inbox_dir/browser.src"
    echo "[#] Notifying $repository_notify_file"
    echo "$REPOSITORY_TMP_ARTIFACT_DIR" > "$repository_notify_file"
done

# Remove our temporary working directory
rm -r "$TMP_DIR"
