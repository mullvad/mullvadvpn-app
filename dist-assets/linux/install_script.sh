#!/usr/bin/env bash
set -eu
systemctl enable mullvad-daemon.service
systemctl start mullvad-daemon.service
