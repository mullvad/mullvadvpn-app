#!/usr/bin/env bash

set -eu
shopt -s nullglob

CODE_SIGNING_KEY_FINGERPRINT="A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
UPLOAD_DIR="$SCRIPT_DIR/upload"

cd "$UPLOAD_DIR"

function rsync_upload {
    local file=$1
    local upload_dir=$2
    rsync -av --mkpath --rsh='ssh -p 1122' "$file" "upload-server-1:$upload_dir/"
}

while true; do
    sleep 10
    for checksums_path in *.sha256; do
        sleep 1

        # Parse the platform name and version out of the filename of the checksums file.
        platform="$(echo "$checksums_path" | cut -d + -f 1)"
        version="$(echo "$checksums_path" | cut -d + -f 3,4 | sed 's/\.sha256//')"
        if ! sha256sum --quiet -c "$checksums_path"; then
            echo "Failed to verify checksums for $version"
            continue
        fi

        if [[ $version == *"-dev-"* ]]; then
            upload_path="$platform/builds"
        else
            upload_path="$platform/releases"
        fi

        files=$(awk '{print $2}' < "$checksums_path")
        for file in $files; do
            file_upload_dir="$upload_path/$version"
            if [[ $platform == "desktop" && ! $file == MullvadVPN-* ]]; then
                file_upload_dir="$file_upload_dir/additional-files"
            elif [[ $platform == "android" && ! $file =~ MullvadVPN-$version(.apk|.play.aab) ]]; then
                file_upload_dir="$file_upload_dir/additional-files"
            fi

            rsync_upload "$file" "$file_upload_dir/" || continue

            if [[ $file == MullvadVPN-* ]]; then
                rm -f "$file.asc"
                gpg -u $CODE_SIGNING_KEY_FINGERPRINT --pinentry-mode loopback --sign --armor --detach-sign "$file"
                rsync_upload "$file.asc" "$file_upload_dir/" || continue
                rm -f "$file.asc"
            fi

            # shellcheck disable=SC2216
            yes | rm "$file"
        done

        # shellcheck disable=SC2216
        yes | rm "$checksums_path"
    done
done
