#!/bin/bash

set -euo pipefail

# shellcheck source=/dev/null
source ~/.bash_profile

if which brew 
then
    echo >&1 "Installing xcodes"
    brew install xcodesorg/made/xcodes
fi
