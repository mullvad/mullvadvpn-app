#!/usr/bin/env bash
set -eu

if which systemctl &> /dev/null; then
    if systemctl status mullvad-daemon &> /dev/null; then
        systemctl stop mullvad-daemon.service
        systemctl disable mullvad-daemon.service
    fi
fi
