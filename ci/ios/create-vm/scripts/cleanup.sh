#!/bin/bash

set -euo pipefail

# shellcheck source=/dev/null
source ~/.bash_profile


# Uninstall rust
# shellcheck source=/dev/null
if [[ -f "${HOME}/.cargo/env" ]]
then
    source "${HOME}/.cargo/env"
    yes | rustup self uninstall
fi

# Uninstall brew (This should also delete all dependencies)
NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/uninstall.sh)"
# Clean up folders that were not automatically removed
sudo rm -rf /opt/homebrew

# Remove the custom profiles
rm -f ~/.zprofile ~/.profile ~/.bash_profile