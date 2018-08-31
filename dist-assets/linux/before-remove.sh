#!/usr/bin/env bash
set -eu

systemctl stop mullvad-daemon.service
systemctl disable mullvad-daemon.service
