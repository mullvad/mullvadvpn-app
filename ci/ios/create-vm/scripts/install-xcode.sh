#!/bin/bash

set -euo pipefail

# shellcheck source=/dev/null
source ~/.bash_profile

echo >&1 "Installing xcode"
xcodes install "${XCODE_VERSION}" --path "${XCODE_SHARED_PATH}/${XCODE_XIP_NAME}" --experimental-unxip
xcodes select "${XCODE_VERSION}"
