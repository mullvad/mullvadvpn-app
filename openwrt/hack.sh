#/usr/bin/env sh

# Run this script on the OpenWRT host before running any Mullvad app binaries.

echo "Hack in progress, pls remain calm "

sleep 1

# Create log directory
mkdir -p /var/cache
printf "."

# Install kernel module for creating TUN devices
{
  opkg update
  opkg install kmod-tun
} > /dev/null
printf "."

sleep 1

echo ""
echo "Mainframe hacked!"
