#!/usr/bin/env bash

UPLOAD_DIR="/home/upload/upload"

set -eu
shopt -s nullglob

cd $UPLOAD_DIR

while true; do
  sleep 10
  for f_checksum in MullvadVPN-*.{deb,rpm,exe,pkg,apk}.sha256; do
    sleep 1
    f="${f_checksum/.sha256/}"
    if ! sha256sum --quiet -c "$f_checksum"; then
      echo "Failed to verify checksum for $f"
      continue
    fi

    version=$(echo $f | sed -Ee 's/MullvadVPN-(.*)(\.exe|\.pkg|_amd64\.deb|_x86_64\.rpm|.apk)/\1/g')
    ssh build.mullvad.net mkdir -p "app/$version" || continue
    scp -pB "$f" build.mullvad.net:app/$version/ || continue

    rm -f "$f.asc"
    gpg -u A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF --pinentry-mode loopback --sign --armor --detach-sign "$f"
    scp -pB "$f.asc" build.mullvad.net:app/$version/ || true
    yes | rm "$f" "$f_checksum" "$f.asc"
  done
done
