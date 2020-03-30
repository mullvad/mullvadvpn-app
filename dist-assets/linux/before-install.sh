#!/usr/bin/env bash
set -eu

if which systemctl &> /dev/null; then
    if systemctl status mullvad-daemon &> /dev/null; then
        if [[ -f /opt/Mullvad\ VPN/resources/mullvad-setup ]]; then
            /opt/Mullvad\ VPN/resources/mullvad-setup prepare-restart
        fi
        systemctl stop mullvad-daemon.service
        systemctl disable mullvad-daemon.service
    fi
fi

rm -f /var/cache/mullvad-vpn/relays.json || true
