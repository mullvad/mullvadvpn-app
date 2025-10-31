#!/usr/bin/env bash

# Start the OpenWRT VM in persistant mode (to actually install & configure stuff once)
#
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

VM=openwrt.qcow2
echo "Starting $VM"

qemu-system-x86_64 -M q35 \
  -drive file="$SCRIPT_DIR/$VM",id=d0,if=none,bus=0,unit=0 \
  -device ide-hd,drive=d0,bus=ide.0 -nic user,hostfwd=tcp::1337-:22 \
  -nographic
