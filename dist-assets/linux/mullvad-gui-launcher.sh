#!/usr/bin/env bash
set -eu

UNPRIVILEGED_USERNS_PATH="/proc/sys/kernel/unprivileged_userns_clone"
if [ -e $UNPRIVILEGED_USERNS_PATH ] && grep -q 0 $UNPRIVILEGED_USERNS_PATH; then
    SANDBOX_FLAG="--no-sandbox"
else
    SANDBOX_FLAG=""
fi

SUPPORTED_COMPOSITORS="sway river Hyprland"
if [ "${XDG_SESSION_TYPE:-""}"  = "wayland" ] && \
    echo " $SUPPORTED_COMPOSITORS " | \
    grep -qi -e " ${XDG_CURRENT_DESKTOP:-""} " -e " ${XDG_SESSION_DESKTOP:-""} "
then
    WAYLAND_FLAGS="--ozone-platform=wayland --enable-features=WaylandWindowDecorations"
else
    WAYLAND_FLAGS=""
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/mullvad-gui" $SANDBOX_FLAG $WAYLAND_FLAGS "$@"
