#!/usr/bin/env bash
set -eu

if which systemctl &> /dev/null && systemctl is-system-running | grep -vq offline &> /dev/null; then
    if systemctl status mullvad-daemon &> /dev/null; then
        /opt/Mullvad\ VPN/resources/mullvad-setup prepare-restart || true
        systemctl stop mullvad-daemon.service
        systemctl disable mullvad-daemon.service
        systemctl disable mullvad-early-boot-blocking.service || true
        cp /var/log/mullvad-vpn/daemon.log /var/log/mullvad-vpn/old-install-daemon.log \
            || echo "Failed to copy old daemon log"
    fi
fi

rm -f /var/cache/mullvad-vpn/relays.json
rm -f /var/cache/mullvad-vpn/api-ip-address.txt
