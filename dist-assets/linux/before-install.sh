#!/usr/bin/env bash
set -eu

function get_home_dirs {
  if [[ -f "/etc/passwd" ]] && command -v cut > /dev/null; then
      cut -d: -f6 /etc/passwd
  fi
}

function clear_gpu_cache {
  local home_dirs
  home_dirs=$(get_home_dirs)

  for home_dir in $home_dirs; do
      local gpu_cache_dir="$home_dir/.config/Mullvad VPN/GPUCache"
      if [[ -d "$gpu_cache_dir" ]]; then
          echo "Clearing GPU cache in $gpu_cache_dir"
          rm -r --interactive=never "$gpu_cache_dir" || \
              echo "Failed to clear GPU cache"
      fi
  done
}

if which systemctl &> /dev/null; then
    if systemctl status mullvad-daemon &> /dev/null; then
        /opt/Mullvad\ VPN/resources/mullvad-setup prepare-restart || true
        systemctl stop mullvad-daemon.service
        systemctl disable mullvad-daemon.service
        systemctl disable mullvad-early-boot-blocking.service || true
        cp /var/log/mullvad-vpn/daemon.log /var/log/mullvad-vpn/old-install-daemon.log \
            || echo "Failed to copy old daemon log"
    fi
fi

# This can be removed when 2022.4 is unsupported. That version is the last version where
# before-remove.sh doesn't kill the GUI on upgrade.
pkill -x "mullvad-gui" || true

# We've had reports of corrumpt GPU caches. Clearing on upgrade will solve these issues.
clear_gpu_cache

rm -f /var/cache/mullvad-vpn/relays.json
rm -f /var/cache/mullvad-vpn/api-ip-address.txt
