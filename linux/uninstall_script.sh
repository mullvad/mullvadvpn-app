#!/bin/bash
set -eux
systemctl stop mullvad-daemon.service
systemctl disable mullvad-daemon.service
