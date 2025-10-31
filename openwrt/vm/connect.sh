#!/usr/bin/env bash

# Connect to a running VM
#
# Based on heavy assumptions, such as the VM already running by the virtue of ./debug.sh.
#
# Depends on: ssh

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

OPENWRT_USER=root

ssh "$OPENWRT_USER@localhost" -p 1337
