#!/usr/bin/env bash
set -eu

function remove_logs_and_cache {
  rm -r --interactive=never /var/log/mullvad-vpn/ || \
    echo "Failed to remove mullvad-vpn logs"
  rm -r --interactive=never /var/cache/mullvad-vpn/ || \
    echo "Failed to remove mullvad-vpn cache"
}

function remove_config {
  rm -r --interactive=never /etc/mullvad-vpn || \
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
    ;;
  # yum remove passes a 0
  "0")
    remove_logs_and_cache
    remove_config
    ;;
esac
