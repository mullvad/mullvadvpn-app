#!/bin/bash

set -euo pipefail

# shellcheck source=/dev/null
source ~/.bash_profile

if command -v brew &>/dev/null
then
    echo >&1 "Installing go@1.19"
    brew install go@1.19
    echo "export PATH='/opt/homebrew/opt/go@1.19/bin:$PATH'" >> ~/.bash_profile
fi