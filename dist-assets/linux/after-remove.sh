#!/usr/bin/env bash
set -eu

function remove_systemd_unit {
  systemctl stop mullvad-daemon.service || \
    echo "Failed to stop mullvad-daemon service"
  systemctl disable mullvad-daemon.service || \
    echo "Failed to disable mullvad-daemon service"
}

function remove_logs_and_cache {
  rm -rf /var/log/mullvad-vpn/ || \
    echo "Failed to remove mullvad-vpn logs"
  rm -rf /var/cache/mullvad-vpn/ || \
    echo "Failed to remove mullvad-vpn cache"
}

function remove_config {
  rm -rf /etc/mullvad-vpn || \
    echo "Failed to remove mullvad-vpn config"
}

# checking what kind of an action is taking place
case $@ in
  # apt purge passes "purge"
  "purge")
    remove_logs_and_cache
    remove_config
    ;;
  # apt remove passes "remove"
  "remove")
    remove_systemd_unit
    ;;
  # yum remove passes a 0
  "0")
    remove_logs_and_cache
    remove_systemd_unit
    remove_config
    ;;
esac
