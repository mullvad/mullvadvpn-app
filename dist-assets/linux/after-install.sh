#!/usr/bin/env bash
set -eu

if which systemctl &> /dev/null; then
    systemctl enable "/opt/Mullvad VPN/init/systemd/mullvad-daemon.service"
    systemctl start mullvad-daemon.service
elif /sbin/init --version | grep upstart &> /dev/null; then
    ln -s "/opt/Mullvad VPN/init/upstart/mullvad-daemon.conf" /etc/init/
    initctl reload-configuration
    start mullvad-daemon
fi
