#!/usr/bin/env bash

UPLOAD_DIR="/home/upload/upload"
BUILD_ARTIFACT_EXTENSIONS="deb|rpm|exe|pkg|apk|aab"

set -eu
shopt -s nullglob

cd $UPLOAD_DIR

while true; do
  sleep 10

  for f_checksums in *.sha256; do
    sleep 1

    version="${f_checksums/+*/}"
    if ! sha256sum --quiet -c "$f_checksums"; then
      echo "Failed to verify checksums for $version"
      continue
    fi

    if [[ $version == *"-dev-"* ]]; then
      upload_path="builds"
    else
      upload_path="releases"
    fi

    files=$(awk '{print $2}' < "$f_checksums")
    for f in $files; do
      if [[ ! $f =~ \.($BUILD_ARTIFACT_EXTENSIONS|asc)$ ]]; then
        upload_path="$upload_path/additional-files"
      fi

      rsync -av --rsh='ssh -p 1122' "$f" "build@releases.mullvad.net:$upload_path/$version/" || continue

      if [[ $f =~ \.($BUILD_ARTIFACT_EXTENSIONS)$ ]]; then
        rm -f "$f.asc"
        gpg -u A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF --pinentry-mode loopback --sign --armor --detach-sign "$f"
        rsync -av --rsh='ssh -p 1122' "$f.asc" "build@releases.mullvad.net:$upload_path/$version/" || continue
        rm -f "$f.asc"
      fi
    done

    # shellcheck disable=SC2216
    yes | rm "${files[@]}" "$f_checksums"
  done
done
