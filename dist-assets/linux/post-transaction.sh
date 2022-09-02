#!/usr/bin/env bash
# This is to mitigate post-uninstall hooks being ran AFTER post-install hooks
# during an upgrade on Fedora.
set -eu

# Repeated enablement of the daemon service will result in the early-boot unit
# being executed when the daemon is already running, which results in the
# firewall rules being applied.
if ! systemctl is-enabled mullvad-daemon; then
    systemctl enable "/usr/lib/systemd/system/mullvad-daemon.service" || true
    systemctl start mullvad-daemon.service || true
    systemctl enable "/usr/lib/systemd/system/mullvad-early-boot-blocking.service" || true
fi
