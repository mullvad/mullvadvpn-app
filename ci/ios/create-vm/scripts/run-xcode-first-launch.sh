#!/bin/bash

set -euo pipefail

# shellcheck source=/dev/null
source ~/.bash_profile

if command -v xcodebuild &>/dev/null
then
    echo >&1 "Running xcodebuild -runFirstLaunch"
    xcodebuild -runFirstLaunch
    echo >&1 "Downloading iOS Simulator runtime, this might take a while"
    xcodebuild -downloadPlatform iOS
fi
