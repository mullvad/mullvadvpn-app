#!/bin/bash

set -euo pipefail

# shellcheck source=/dev/null
source ~/.bash_profile

if which xcodebuild
then
    echo >&1 "Running xcodebuild -runFirstLaunch"
    xcodebuild -runFirstLaunch
    echo >&1 "Downloading iOS Simulator runtime, this might take a while"
    xcodebuild -downloadPlatform iOS 
fi

# Xcode is needed in order to run swiftformat or swiftlint
brew install swiftformat swiftlint