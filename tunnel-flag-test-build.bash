#!/bin/bash

set -ex

CRATES="\
    mullvad-cli \
    mullvad-daemon \
    mullvad-ipc-client \
    mullvad-paths \
    mullvad-problem-report \
    mullvad-rpc \
    mullvad-tests \
    mullvad-types \
    talpid-core \
    talpid-ipc \
    talpid-openvpn-plugin \
    talpid-types \
"

for crate in $CRATES; do
    pushd "$crate"

    if grep '\[features\]' -A2 Cargo.toml | grep '^openvpn' &> /dev/null; then
        cargo build
        cargo build --features openvpn
        cargo build --features wireguard
        cargo build --features openvpn,wireguard
    else
        cargo build
    fi

    popd
done
