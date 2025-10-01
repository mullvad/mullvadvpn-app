#!/usr/bin/env bash
set -eu

chmod u+s "/usr/bin/mullvad-exclude"

systemctl enable "/usr/lib/systemd/system/mullvad-daemon.service"
systemctl start mullvad-daemon.service || echo "Failed to start mullvad-daemon.service"
systemctl enable "/usr/lib/systemd/system/mullvad-early-boot-blocking.service"

# Detect if the system is using apparmor. Valid exit codes are: 0, 1, 2 (man aa-status).
function exists() {
    command -v "$1" >/dev/null 2>&1
}

if exists aa-status; then
    # If that's the case, install our apparmor profile and try to reload apparmor.
    # The apparmor profile allow Electron sandbox to work.
    # This disables user namespace restrictions.

    echo "Creating apparmor profile"
    cp /opt/Mullvad\ VPN/resources/apparmor_mullvad /etc/apparmor.d/mullvad
    apparmor_parser -r /etc/apparmor.d/mullvad || echo "Failed to reload apparmor profile"
fi
