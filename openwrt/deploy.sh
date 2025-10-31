#!/usr/bin/env bash

# Assuming that an OpenWRT x86 VM is running on localhost and forwards ssh to local port 1337.

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

ARTIFACTS="$SCRIPT_DIR/../target/x86_64-unknown-linux-musl/release"

OPENWRT_USER=root
OPENWRT_DIR=/root

# Add these binaries to `/usr/bin` so that they are in $PATH.
scp -P 1337 "$ARTIFACTS/mullvad" "$OPENWRT_USER@localhost:/usr/bin"
scp -P 1337 "$ARTIFACTS/mullvad-daemon" "$OPENWRT_USER@localhost:/usr/sbin"

# The hack.sh script are workarounds for known issues with mullvad on OpenWRT. See `known-issues.txt` for more info.
scp -P 1337 "$SCRIPT_DIR/hack.sh" "$OPENWRT_USER@localhost:$OPENWRT_DIR"
