#!/usr/bin/env bash

UPLOAD_DIR="/home/upload/upload"

set -eu
shopt -s nullglob

cd $UPLOAD_DIR

while true;
do
  sleep 10
  for f in MullvadVPN-*;
  do
    sleep 10
    version=$(echo $f | sed -Ee 's/MullvadVPN-(.*)(\.exe|\.pkg|_amd64\.deb|_x86_64\.rpm)/\1/g')
    ssh build.mullvad.net mkdir -p "app/$version" || continue
    scp -B "$f" build.mullvad.net:app/$version/ || continue

    rm -f "$f.asc"
    gpg -u A1198702FC3E0A09A9AE5B75D5A1D4F266DE8DDF --pinentry-mode loopback --sign --armor --detach-sign "$f"
    scp -B "$f.asc" build.mullvad.net:app/$version/ || true
    yes | rm "$f" "$f.asc"
  done
done
