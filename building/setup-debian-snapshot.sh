#!/usr/bin/env bash

set -eu

. /etc/os-release
rm -f /etc/apt/sources.list.d/debian.sources
# check-valid-until=no is required since the Release files in old snapshots have expired.
printf '%s\n' \
    "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/$DEBIAN_SNAPSHOT $VERSION_CODENAME main" \
    "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/$DEBIAN_SNAPSHOT $VERSION_CODENAME-updates main" \
    "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian-security/$DEBIAN_SNAPSHOT $VERSION_CODENAME-security main" \
    > /etc/apt/sources.list
