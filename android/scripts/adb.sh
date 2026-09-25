#!/usr/bin/env bash

set -eu

PACKAGE=${PACKAGE:-net.mullvad.mullvadvpn}
LOCAL_SOCKET_PATH=${LOCAL_SOCKET_PATH:-${XDG_RUNTIME_DIR:-/tmp}/mullvad-android.sock}

case ${1:-} in
    start-app)
        adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1
        ;;
    stop-app)
        adb shell am force-stop "$PACKAGE"
        ;;
    enable-rpc)
        adb shell run-as "$PACKAGE" chmod o+x .
        adb forward "localfilesystem:$LOCAL_SOCKET_PATH" "localfilesystem:/data/data/$PACKAGE/no_backup/rpc-socket"
        printf 'export MULLVAD_RPC_SOCKET_PATH=%q\n' "$LOCAL_SOCKET_PATH"
        ;;
    disable-rpc)
        adb shell run-as "$PACKAGE" chmod o-x .
        adb forward --remove "localfilesystem:$LOCAL_SOCKET_PATH"
        printf 'unset MULLVAD_RPC_SOCKET_PATH\n'
        ;;
    *)
        echo "Usage: $0 start-app|stop-app|enable-rpc|disable-rpc"
        exit 1
        ;;
esac
