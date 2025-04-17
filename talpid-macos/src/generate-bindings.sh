#!/usr/bin/env bash

# This generates new bindings from 'proc_info.h'.
# bindgen is required: cargo install bindgen-cli

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

MACOS_SDK_PATH="$(xcrun --sdk macosx --show-sdk-path)"
PROC_INFO_PATH="$MACOS_SDK_PATH/usr/include/sys/proc_info.h"

bindgen "$PROC_INFO_PATH" -o ./bindings.rs \
    --allowlist-item "^PROC_PIDFDVNODEPATHINFO" \
    --allowlist-item "^PROX_FDTYPE_VNODE" \
    --allowlist-item "^vnode_fdinfowithpath"
