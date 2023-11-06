#!/bin/bash

set -euo pipefail

# shellcheck source=/dev/null
source ~/.bash_profile

if command -v brew &>/dev/null
then
    echo "Installing xcodes"
    brew install xcodesorg/made/xcodes
    echo "Installing xcodes"
    brew install bash
fi
