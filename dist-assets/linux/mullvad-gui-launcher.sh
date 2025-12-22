#!/usr/bin/env bash
set -eu

UNPRIVILEGED_USERNS_PATH="/proc/sys/kernel/unprivileged_userns_clone"
if [ -e $UNPRIVILEGED_USERNS_PATH ] && grep -q 0 $UNPRIVILEGED_USERNS_PATH; then
    SANDBOX_FLAG="--no-sandbox"
else
    SANDBOX_FLAG=""
fi

if [ "${XDG_SESSION_TYPE:-""}"  = "wayland" ]; then
    # If running wayland ensure a supported compositor is used,
    # otherwise force use of X11.
    WAYLAND_SUPPORTED_COMPOSITORS="sway river Hyprland niri"
    if echo " $WAYLAND_SUPPORTED_COMPOSITORS " | grep -qi -e " ${XDG_CURRENT_DESKTOP:-""} " -e " ${XDG_SESSION_DESKTOP:-""} "
    then
        COMPOSITOR_FLAGS=( "--ozone-platform=wayland" "--enable-features=WaylandWindowDecorations" )
    else
        COMPOSITOR_FLAGS=( "--ozone-platform=x11" )
    fi
else
    # If not running wayland then we do not need to set any flags.
    COMPOSITOR_FLAGS=()
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/mullvad-gui" "$SANDBOX_FLAG" "${COMPOSITOR_FLAGS[@]}" "$@"
