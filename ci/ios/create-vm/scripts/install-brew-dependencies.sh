#!/bin/bash

set -euo pipefail

# shellcheck source=/dev/null
source ~/.bash_profile

if command -v brew &>/dev/null
then
    echo >&1 "Installing xcodes"
    brew install xcodesorg/made/xcodes
fi
