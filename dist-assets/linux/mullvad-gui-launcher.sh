#!/usr/bin/env bash
set -eu

UNPRIVILEGED_USERNS_PATH="/proc/sys/kernel/unprivileged_userns_clone"
if [ -e $UNPRIVILEGED_USERNS_PATH ] && grep -q 0 $UNPRIVILEGED_USERNS_PATH; then
    SANDBOX_FLAG="--no-sandbox"
elif command -v lsb_release > /dev/null && \
    [[ "$(lsb_release -i | awk -F : '{print $2}' | xargs echo)" == "Ubuntu" ]] && \
    [[ "$(lsb_release -r | awk -F : '{print $2}' | xargs echo)" == "21.10" ]]; then
    SANDBOX_FLAG="--no-sandbox"
else
    SANDBOX_FLAG=""
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/mullvad-gui" $SANDBOX_FLAG "$@"
