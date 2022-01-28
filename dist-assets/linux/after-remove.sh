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

  # Remove app settings and auto-launcher for all users. This doesn't respect XDG_CONFIG_HOME due
  # to the complexity required.
  if [[ -f "/etc/passwd" ]] && command -v cut > /dev/null; then
      local home_dirs
      home_dirs=$(cut -d: -f6 /etc/passwd)
      for home_dir in $home_dirs; do
          local mullvad_dir="$home_dir/.config/Mullvad VPN"
          if [[ -d "$mullvad_dir" ]]; then
              echo "Removing mullvad-vpn app settings from $mullvad_dir"
              rm -r --interactive=never "$mullvad_dir" || \
                  echo "Failed to remove mullvad-vpn app settings"
          fi

          local autostart_path="$home_dir/.config/autostart/mullvad-vpn.desktop"
          # mullvad-vpn.desktop can be both a file or a symlink.
          if [[ -f "$autostart_path" || -L "$autostart_path" ]]; then
              echo "Removing mullvad-vpn app autostart file $autostart_path"
              rm --interactive=never "$autostart_path" || \
                  echo "Failed to remove mullvad-vpn autostart file"
          fi
      done
  fi
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
