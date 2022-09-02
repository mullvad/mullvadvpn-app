#!/usr/bin/env bash
# This is to mitigate post-uninstall hooks being ran AFTER post-install hooks
# during an upgrade on Fedora.
set -eu
systemctl enable "/etc/systemd/system/mullvad-daemon.service" || true
systemctl start mullvad-daemon.service || true
systemctl enable "/etc/systemd/system/mullvad-early-boot-blocking.service" || true
