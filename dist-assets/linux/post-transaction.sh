#!/usr/bin/env bash
# This is to mitigate post-uninstall hooks being ran AFTER post-install hooks
# during an upgrade on Fedora.
set -eu
systemctl enable "/opt/Mullvad VPN/resources/mullvad-daemon.service" || true
systemctl start mullvad-daemon.service || true
