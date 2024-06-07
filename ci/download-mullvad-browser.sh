#!/usr/bin/bash -e

BROWSER_RELEASES=("alpha")
# Uncomment when stable is available
#BROWSER_RELEASES=("alpha" "stable")
OUTPUT_DIR=/tmp/mullvad-browser-download


function usage() {
    echo "Usage: $0 <notify_file>"
    echo
    echo "This script downloads, verifies, and notifies about Mullvad browser packages."
    echo
    echo "Arguments:"
    echo "  <notify_file>   The path to the file where notifications of updated packages will be written."
    echo
    echo "Options:"
    echo "  --help          Show this help message and exit."
    exit 1
}


function main() {
	# mullvad-browser-alpha.deb
	PACKAGE_FILENAME=$1
	PACKAGE_URL=https://cdn.mullvad.net/browser/$PACKAGE_FILENAME
	SIGNATURE_URL=$PACKAGE_URL.asc

	echo "[#] Downloading $PACKAGE_FILENAME"
	if ! wget --quiet --show-progress $PACKAGE_URL; then
		echo "[!] Failed to download $PACKAGE_URL"
		exit 1
	fi

	echo "[#] Downloading $PACKAGE_FILENAME signature"
	if ! wget --quiet --show-progress $SIGNATURE_URL; then
		echo "[!] Failed to download $SIGNATURE_URL"
		exit 1
	fi

	echo "[#] Verifying $PACKAGE_FILENAME signature"
	if ! gpg --verify $PACKAGE_FILENAME.asc; then
		echo "[!] Failed to verify signature"
		exit 1
	fi

	# Check if the deb package has changed since last time
	# Handle the bootstrap problem by checking if the "output file" even exists and just moving on if it doesn't
	if [[ -f $OUTPUT_DIR/$PACKAGE_FILENAME ]] && cmp $PACKAGE_FILENAME $OUTPUT_DIR/$PACKAGE_FILENAME; then
		echo "[#] $PACKAGE_FILENAME has not changed"
		return
	fi
	echo "[#] $PACKAGE_FILENAME has changed"

	echo "[#] Moving $PACKAGE_FILENAME file to $OUTPUT_DIR"
	mv $PACKAGE_FILENAME $OUTPUT_DIR/

	echo "[#] Notifying $NOTIFY_FILE"
	echo "$OUTPUT_DIR/$PACKAGE_FILENAME" >> $NOTIFY_FILE
}

if [[ $# == 0 ]] || [[ $1 == "--help" ]]; then
	usage
fi


NOTIFY_FILE=$(readlink -f $1)
if [[ -z $NOTIFY_FILE ]]; then
	echo "Please provide the output path as the first argument"
	exit 1
fi 

if ! [[ -d $OUTPUT_DIR ]]; then
	echo "[#] Creating $OUTPUT_DIR"
	mkdir -p $OUTPUT_DIR
fi

# Prepare working area
WORKDIR=$(mktemp -q -d )
pushd $WORKDIR > /dev/null
trap "{ rm -r $WORKDIR; }" EXIT

for release in ${BROWSER_RELEASES[@]}; do
	main mullvad-browser-$release.deb
	# Uncomment when rpm is available
	#main mullvad-browser-$release.rpm
done

