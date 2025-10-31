#!/usr/bin/env bash

# This script will quickly bootstrap an OpenWRT 24.10 x86_64 virtual machine
# which can be used for debugging purposes.
#
# This script depend on: wget gunzip qemu

echo "Downloading OpenWRT image .."

wget https://downloads.openwrt.org/releases/24.10.4/targets/x86/64/openwrt-24.10.4-x86-64-generic-ext4-combined-efi.img.gz
gunzip openwrt-24.10.4-x86-64-generic-ext4-combined-efi.img.gz
qemu-img convert -f raw -O qcow2 openwrt-24.10.4-x86-64-generic-ext4-combined-efi.img openwrt.qcow2
qemu-img resize openwrt.qcow2 1024M

echo "Done"
