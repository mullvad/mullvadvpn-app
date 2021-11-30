#!/usr/bin/env bash
set -eu

is_number_re='^[0-9]+$'
# Check if we're running during an upgrade step on Fedora
# https://fedoraproject.org/wiki/Packaging:Scriptlets#Syntax
if [[ "$1" =~ $is_number_re ]] && [ $1 -gt 0 ]; then
    echo not running
    exit 0;
fi

if [[ "$1" == "upgrade" ]]; then
    echo not running
    exit 0;
fi

if which systemctl &> /dev/null; then
    # the user might've disabled or stopped the service themselves already
    systemctl stop mullvad-daemon.service || true
    systemctl disable mullvad-daemon.service || true
elif /sbin/init --version | grep upstart &> /dev/null; then
    stop mullvad-daemon
    rm -f /etc/init/mullvad-daemon.conf
fi

pkill -x "mullvad-gui" || true

/opt/Mullvad\ VPN/resources/mullvad-setup reset-firewall || echo "Failed to reset firewall"
/opt/Mullvad\ VPN/resources/mullvad-setup remove-wireguard-key || echo "Failed to remove leftover WireGuard key"
