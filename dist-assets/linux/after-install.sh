#!/usr/bin/env bash
set -eu

chmod u+s "/usr/bin/mullvad-exclude"

systemctl enable "/usr/lib/systemd/system/mullvad-daemon.service"
systemctl start mullvad-daemon.service
systemctl enable "/usr/lib/systemd/system/mullvad-early-boot-blocking.service"
