#!/usr/bin/env bash
set -eu

chmod u+s "/usr/bin/mullvad-exclude"

if which systemctl &> /dev/null; then
    systemctl enable "/usr/lib/systemd/system/mullvad-daemon.service"
    systemctl start mullvad-daemon.service || echo "Failed to start mullvad-daemon.service"
    systemctl enable "/usr/lib/systemd/system/mullvad-early-boot-blocking.service"
else
    echo "systemd does not exist on this system. The daemon service will not be auto-started!"
fi

# Check if the system supports a new-enough AppArmor version.
function supported_apparmor() {
    [[ -e /etc/apparmor.d/abi/4.0 ]]
}

if supported_apparmor; then
    # Install our AppArmor profile and try to reload AppArmor.
    # The AppArmor profile allow Electron sandbox to work.
    # This disables user namespace restrictions.
    echo "Creating apparmor profile"
    cp /opt/Mullvad\ VPN/resources/apparmor_mullvad /etc/apparmor.d/mullvad
    apparmor_parser -r /etc/apparmor.d/mullvad || echo "Failed to reload apparmor profile"
fi
