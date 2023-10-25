#!/bin/bash

set -euo pipefail

if which brew
then
    echo >&1 "brew is already installed, nothing to do here"
    exit 0
fi

echo >&1 "installing brew"
NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.bash_profile
eval "$(/opt/homebrew/bin/brew shellenv)"

# shellcheck source=/dev/null
source ~/.bash_profile
brew update
