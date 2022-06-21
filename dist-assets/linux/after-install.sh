#!/usr/bin/env bash
set -eu

chmod u+s "/usr/bin/mullvad-exclude"

systemctl enable "/opt/Mullvad VPN/resources/mullvad-daemon.service"
systemctl start mullvad-daemon.service
