#!/bin/bash
set -eux
systemctl enable mullvad-daemon.service
systemctl start mullvad-daemon.service
