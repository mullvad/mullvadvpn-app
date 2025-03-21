#!/usr/bin/env bash

set -eu
shopt -s nullglob globstar

CODE_SIGNING_KEY_FINGERPRINT="A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# shellcheck source=ci/buildserver-config.sh
source "$SCRIPT_DIR/buildserver-config.sh"

cd "$UPLOAD_DIR"

function rsync_upload {
    local file=$1
    local upload_dir=$2
    for server in "${PRODUCTION_UPLOAD_SERVERS[@]}"; do
        echo "Uploading $file to $server:$upload_dir"
        rsync -av --mkpath --rsh='ssh -p 1122' "$file" "$server:$upload_dir/"
    done
}

while true; do
    sleep 10
    for checksums_path in **/*.sha256; do
        sleep 1

        checksums_dir=$(dirname "$checksums_path")
        checksums_filename=$(basename "$checksums_path")

        # Parse the platform name and version out of the filename of the checksums file.
        platform="$(echo "$checksums_filename" | cut -d + -f 1)"
        version="$(echo "$checksums_filename" | cut -d + -f 3,4 | sed 's/\.sha256//')"
        if ! (cd "$checksums_dir" && sha256sum --quiet -c "$checksums_filename"); then
            echo "Failed to verify checksums for $version"
            continue
        fi

        if [[ "$platform" == "installer-downloader" ]]; then
            upload_path="desktop/installer-downloader"
        elif [[ $version == *"-dev-"* ]]; then
            upload_path="$platform/builds"
        else
            upload_path="$platform/releases"
        fi

        # Read all files listed in the checksum file at $checksums_path into an array.
        # sed is used to trim surrounding whitespace and asterisks from filenames.
        readarray -t files < <(cut -f 2- -d ' ' < "$checksums_path" | sed 's/^[ \t\*]*\(.*\)[ \t]*$/\1/')
        for filename in "${files[@]}"; do
            file="$checksums_dir/$filename"

            file_upload_dir="$upload_path/$version"
            if [[ $platform == "desktop" && ! $filename == MullvadVPN-* ]]; then
                file_upload_dir="$file_upload_dir/additional-files"
            elif [[ $platform == "android" && ! $filename =~ MullvadVPN-"$version"(.apk|.play.apk|.play.aab) ]]; then
                file_upload_dir="$file_upload_dir/additional-files"
            fi

            rsync_upload "$file" "$file_upload_dir/" || continue

            if [[ $filename == MullvadVPN-* || $filename == Install* ]]; then
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
