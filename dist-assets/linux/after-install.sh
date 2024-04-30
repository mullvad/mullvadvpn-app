#!/usr/bin/env bash
set -eu

chmod u+s "/usr/bin/mullvad-exclude"

systemctl enable "/usr/lib/systemd/system/mullvad-daemon.service"
systemctl start mullvad-daemon.service || echo "Failed to start mullvad-daemon.service"
systemctl enable "/usr/lib/systemd/system/mullvad-early-boot-blocking.service"

# return 0 if version $1 is greater than or equal to $2
function version_is_ge {
    [ "$1" = "$2" ] && return 0
    printf '%s\n' "$2" "$1" | sort -C -V
}

# Ubuntu 24.04 or newer: Install apparmor profile to allow Electron sandbox to work
# This disables user namespace restrictions
os=$(grep -oP '^ID=\K.+' /etc/os-release | tr -d '"')
version=$(grep -oP '^VERSION_ID=\K.+' /etc/os-release | tr -d '"')

if [[ "$os" == "ubuntu" ]] && version_is_ge "$version" "24.04"; then
    echo "Creating apparmor profile"
    cp /opt/Mullvad\ VPN/resources/apparmor_mullvad /etc/apparmor.d/mullvad
    apparmor_parser -r /etc/apparmor.d/mullvad || echo "Failed to reload apparmor profile"
fi
