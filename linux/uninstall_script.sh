#!/bin/bash
set -eu

UNIT_FILE_BASENAME="mullvad-daemon.service"

function main
{
    disable_service
    remove_unit_file
}

function disable_service
{
  systemctl stop $UNIT_FILE_BASENAME
  systemctl disable $UNIT_FILE_BASENAME
}

function remove_unit_file
{
  # /etc/systemd/system is all but a hardcoded path that all systemd
  # installations have. Unlike /usr/lib/systemd/system, it is highly unlikely
  # that the OS would overwrite unit files here.
  rm /etc/systemd/system/$UNIT_FILE_BASENAME
}

main
