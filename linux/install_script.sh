#!/bin/bash
set -eu

UNIT_FILE_BASENAME="mullvad-daemon.service"

function main
{
  write_unit_file
  enable_service
}

function enable_service
{
  systemctl enable $UNIT_FILE_BASENAME
  systemctl start $UNIT_FILE_BASENAME
}

function write_unit_file
{
  # /etc/systemd/system is all but a hardcoded path that all systemd
  # installations have. Unlike /usr/lib/systemd/system, it is highly unlikely
  # that the OS would overwrite unit files here.
  unit_file > /etc/systemd/system/$UNIT_FILE_BASENAME
}

function unit_file
{
  cat <<EOF
  [Unit]
  Description=Mullvad daemon
  Wants=network.target

  [Service]
  ExecStart="/opt/Mullvad VPN/resources/mullvad-daemon"

  [Install]
  WantedBy=multi-user.target
EOF
}

main
