#!/usr/bin/env bash
set -eu

should_remove_systemd_unit=''
should_clear_cache_and_logs=''

# checking what kind of an action is taking place
case $@ in
  # apt purge passes "purge"
  "purge")
    should_clear_cache_and_logs='1'
    ;;
  # apt remove passes "remove"
  "remove")
    should_remove_systemd_unit='1'
    ;;
  # yum remove passes a 0
  "0")
    should_clear_cache_and_logs='1'
    should_remove_systemd_unit='1'
    ;;
esac

if ! [ -z "$should_remove_systemd_unit" ]; then
  systemctl stop mullvad-daemon.service
  systemctl disable mullvad-daemon.service
fi

if ! [ -z "$should_clear_cache_and_logs" ]; then
  rm -rf /var/log/mullvad-daemon/
  rm -rf /var/cache/mullvad-daemon/
fi
