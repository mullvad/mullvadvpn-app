#!/usr/bin/env bash
set -eu

if which systemctl &> /dev/null; then
    systemctl enable mullvad-daemon.service
    systemctl start mullvad-daemon.service
elif /sbin/init --version | grep upstart &> /dev/null; then
    start mullvad-daemon
fi
