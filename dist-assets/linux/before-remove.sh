#!/usr/bin/env bash
set -eu

if which systemctl &> /dev/null; then
    systemctl stop mullvad-daemon.service
    systemctl disable mullvad-daemon.service
elif /sbin/init --version | grep upstart &> /dev/null; then
    stop mullvad-daemon
    rm -f /etc/init/mullvad-daemon.conf
fi
