#!/usr/bin/env bash

UPLOAD_DIR="/home/upload/upload"
BUILD_ARTIFACT_EXTENSIONS="deb|rpm|exe|pkg|apk|aab"

set -eu
shopt -s nullglob

cd $UPLOAD_DIR

while true; do
  sleep 10

  for checksums_filename in *.sha256; do
    sleep 1

    version="${checksums_filename/+*/}"
    if ! sha256sum --quiet -c "$checksums_filename"; then
      echo "Failed to verify checksums for $version"
      continue
    fi

    if [[ $version == *"-dev-"* ]]; then
      upload_path="builds"
    else
      upload_path="releases"
    fi

    files=$(awk '{print $2}' < "$checksums_filename")
    for file in $files; do
      file_upload_dir="$upload_path/$version"
      if [[ ! $file =~ \.($BUILD_ARTIFACT_EXTENSIONS|asc)$ ]]; then
        file_upload_dir="$file_upload_dir/additional-files"
      fi

      rsync -av --rsh='ssh -p 1122' "$file" "build@releases.mullvad.net:$file_upload_dir/" || continue

      if [[ $file =~ \.($BUILD_ARTIFACT_EXTENSIONS)$ ]]; then
        rm -f "$file.asc"
        gpg -u A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF --pinentry-mode loopback --sign --armor --detach-sign "$file"
        rsync -av --rsh='ssh -p 1122' "$file.asc" "build@releases.mullvad.net:$file_upload_dir/" || continue
        rm -f "$file.asc"
      fi
    done

    # shellcheck disable=SC2216
    yes | rm "${files[@]}" "$checksums_filename"
  done
done
