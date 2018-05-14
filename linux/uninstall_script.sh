#!/usr/bin/env bash
set -ux
systemctl stop mullvad-daemon.service
systemctl disable mullvad-daemon.service
exit 0
