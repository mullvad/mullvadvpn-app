#!/usr/bin/env bash

# This generates new bindings from 'proc_info.h'.
# bindgen is required: cargo install bindgen-cli

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

MACOS_SDK_PATH="$(xcrun --sdk macosx --show-sdk-path)"
PROC_INFO_PATH="$MACOS_SDK_PATH/usr/include/sys/proc_info.h"

cp ./apsl-header ./bindings.rs

bindgen "$PROC_INFO_PATH" \
    --allowlist-item "^PROC_PIDFDVNODEPATHINFO" \
    --allowlist-item "^PROX_FDTYPE_VNODE" \
    --allowlist-item "^vnode_fdinfowithpath" \
    >> ./bindings.rs
