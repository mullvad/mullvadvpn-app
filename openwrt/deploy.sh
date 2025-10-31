#!/usr/bin/env bash

# Assuming that an OpenWRT x86 VM is running on localhost and forwards ssh to local port 1337.
#
# Run package.sh before running this script to creat the .ipk archive :-)

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

if [ -z "$1" ]; then
    echo "No .ipk was supplied"
    echo "Usage: $0 <mullvad_version.arch>.ipk"
    exit 1
fi

OPENWRT_USER=root
OPENWRT_DIR=/root

scp -P 1337 "$1" "$OPENWRT_USER@localhost:$OPENWRT_DIR"
