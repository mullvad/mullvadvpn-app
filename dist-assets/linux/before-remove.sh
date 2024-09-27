#!/usr/bin/env bash
set -eu

echo "Running before-remove.sh"

is_number_re='^[0-9]+$'
# Check if we're running during an upgrade step on Fedora
# https://fedoraproject.org/wiki/Packaging:Scriptlets#Syntax
if [[ "$1" =~ $is_number_re ]] && [ "$1" -gt 0 ]; then
    exit 0;
fi

if [[ "$1" == "upgrade" ]]; then
    exit 0;
fi

# the user might've disabled or stopped the service themselves already
if which systemctl &> /dev/null; then
    systemctl stop mullvad-daemon.service || true
    systemctl disable mullvad-daemon.service || true
    systemctl stop mullvad-early-boot-blocking.service || true
    systemctl disable mullvad-early-boot-blocking.service || true
fi

/opt/Mullvad\ VPN/resources/mullvad-setup reset-firewall || echo "Failed to reset firewall"
/opt/Mullvad\ VPN/resources/mullvad-setup remove-device || echo "Failed to remove device from account"
