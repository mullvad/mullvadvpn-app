#!/bin/bash 
set -euo pipefail

# shellcheck source=/dev/null
source ~/.bash_profile

if ! command -v rustup &>/dev/null
then
    echo >&1 "Installing rustup"
    # Install rustup and automatically accept the prompt
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y

    # shellcheck source=/dev/null
    source "${HOME}/.cargo/env"
    echo "source ~/.cargo/env" >> ~/.bash_profile

    echo >&1 "Installing rustup targets"
    rustup target add aarch64-apple-ios-sim aarch64-apple-ios
fi