#!/usr/bin/env bash

set -ux

function uninstall_linux {
    echo "Uninstalling for Linux"
    sudo rm -r ~/.cache/mullvad-daemon \
        ~/.cache/mullvad-daemon/ \
        ~/.config/MullvadVPN/ \
        /root/.cache/mullvad-daemon/ \
        /root/.config/mullvad-daemon
}


function uninstall_macos {
    echo "Uninstalling for macOS"
    sudo rm -r /Applications/MullvadVPN.app \
        ~/Library/Logs/MullvadVPN \
        ~/Library/Caches/mullvad-daemon \
        ~/Library/Application\ Support/mullvad-daemon \
        ~/Library/Application\ Support/MullvadVPN
}

echo "Uninstalling Mullvad VPN"
case "$(uname -s)" in
    Linux*)  uninstall_linux;;
    Darwin*) uninstall_macos;;
    *)       echo "Unsupported platform"; exit 1
esac

