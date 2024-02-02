#!/usr/bin/env bash
set -e

BASE_URL=https://api.crowdin.com/api/project/mullvad-app
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR=$( dirname "$SCRIPT_DIR" )
LOCALE_DIR="$ROOT_DIR/locales"

if [ $# -ne 1 ]; then
    echo "Usage: $0 [upload|export|download]"
    exit 1
elif [ -z "$CROWDIN_API_KEY" ]; then
    echo "Need to set environment variable CROWDIN_API_KEY"
    exit 1
fi

mode=$1

function upload_pot {
    curl \
        -F "files[/messages.pot]=@$LOCALE_DIR/messages.pot" \
        -F "files[/relay-locations.pot]=@$LOCALE_DIR/relay-locations.pot" \
        $BASE_URL/update-file?key="$CROWDIN_API_KEY"
}

function export_translations {
    curl \
        $BASE_URL/export?key="$CROWDIN_API_KEY"
}

function download_translations {
    wget \
        --content-disposition \
        $BASE_URL/download/all.zip?key="$CROWDIN_API_KEY"
    unzip -o all.zip -d "$LOCALE_DIR"
    find "$LOCALE_DIR" -type d -exec chmod 755 {} \;
    find "$LOCALE_DIR" -type f -exec chmod 644 {} \;
    rm all.zip
}

if [[ $mode == "upload" ]]; then
    upload_pot
elif [[ $mode == "export" ]]; then
    export_translations
elif [[ $mode == "download" ]]; then
    download_translations
else
    echo "'$mode' is not a valid mode"
    echo "Usage: $0 [upload|export|download]"
    exit 1
fi
