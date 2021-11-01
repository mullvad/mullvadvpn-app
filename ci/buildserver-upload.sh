#!/usr/bin/env bash

UPLOAD_DIR="/home/upload/upload"

set -eu
shopt -s nullglob

cd $UPLOAD_DIR

while true; do
  sleep 10
  for f_checksum in MullvadVPN-*.{deb,rpm,exe,pkg,apk,aab}.sha256; do
    sleep 1
    f="${f_checksum/.sha256/}"
    if ! sha256sum --quiet -c "$f_checksum"; then
      echo "Failed to verify checksum for $f"
      continue
    fi

    version=$(echo "$f" | sed -Ee 's/MullvadVPN-(.*)(\.exe|\.pkg|_amd64\.deb|_x86_64\.rpm|\.apk|\.aab)/\1/g')
    if [[ $version == *"-dev-"* ]]; then
        upload_path="builds"
    else
        upload_path="releases"
    fi

    ssh build.mullvad.net mkdir -p "app/$version" || continue
    scp -pB "$f" build.mullvad.net:app/$version/ || continue
    rsync -av --rsh='ssh -p 1122' "$f" "build@releases.mullvad.net:$upload_path/$version/" || continue

    rm -f "$f.asc"
    gpg -u A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF --pinentry-mode loopback --sign --armor --detach-sign "$f"
    scp -pB "$f.asc" build.mullvad.net:app/$version/ || true
    rsync -av --rsh='ssh -p 1122' "$f.asc" "build@releases.mullvad.net:$upload_path/$version/" || continue
    yes | rm "$f" "$f_checksum" "$f.asc"
  done
  for f_checksum in pdb-*.sha256; do
    sleep 1
    f="${f_checksum/.sha256/}"
    if ! sha256sum --quiet -c "$f_checksum"; then
      echo "Failed to verify checksum for $f"
      continue
    fi

    ssh build.mullvad.net mkdir -p app/pdb || continue
    scp -pB "$f" build.mullvad.net:app/pdb/ || continue

    yes | rm "$f" "$f_checksum"
  done
done
