#!/bin/bash

set -ex

OUTFILE="mullvad-portable.raw"
DESTDIR="./rootfs"

cd "$(dirname "$0")"

rm -rf "$DESTDIR" "$OUTFILE"
mkdir "$DESTDIR"

mkdir -p \
  "$DESTDIR/etc" \
  "$DESTDIR/usr/bin" \
  "$DESTDIR/usr/lib/systemd/system" \
  "$DESTDIR/var/log" \
  "$DESTDIR/var/tmp" \
  "$DESTDIR/proc" \
  "$DESTDIR/sys" \
  "$DESTDIR/dev" \
  "$DESTDIR/run" \
  "$DESTDIR/tmp"

# TODO: compile daemon statically
cp mullvad-daemon-static "$DESTDIR/usr/bin/mullvad-daemon"

cp mullvad-daemon.service "$DESTDIR/usr/lib/systemd/system/mullvad-portable.service"

cp /usr/lib/os-release "$DESTDIR/usr/lib/os-release"
cat >> "$DESTDIR/usr/lib/os-release" <<EOF
PORTABLE_PRETTY_NAME="Mullvad Daemon (portable)"
EOF

touch "$DESTDIR/etc/resolv.conf"
touch "$DESTDIR/etc/machine-id"

systemd-repart --definitions repart.d --empty=create --size=auto --copy-source "$DESTDIR" mullvad-portable.raw

