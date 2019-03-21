#!/usr/bin/env bash
set -eu

if which systemctl &> /dev/null; then
    if systemctl status mullvad-daemon &> /dev/null; then
        systemctl stop mullvad-daemon.service
        systemctl disable mullvad-daemon.service
    fi
fi

#TODO: Remove after releasing 2019.2
rm /var/cache/mullvad-vpn/relays.json || true
