#!/usr/bin/env bash
set -eu

ln -sf "/opt/Mullvad VPN/resources/mullvad-problem-report" /usr/bin/mullvad-problem-report

if which systemctl &> /dev/null; then
    systemctl enable "/opt/Mullvad VPN/resources/mullvad-daemon.service"
    systemctl start mullvad-daemon.service
elif /sbin/init --version | grep upstart &> /dev/null; then
    ln -s "/opt/Mullvad VPN/resources/mullvad-daemon.conf" /etc/init/
    initctl reload-configuration
    start mullvad-daemon
fi
