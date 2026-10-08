#!/usr/bin/env bash

set -eu

CARGO_CYCLONEDX_VERSION=0.5.9
CARGO_CYCLONEDX_SHA256=5d162f67705f0f5038759d73bf546a083bf30e8677c2e944b416bca48d9d69a8
CARGO_CYCLONEDX_URL=https://static.crates.io/crates/cargo-cyclonedx/cargo-cyclonedx-$CARGO_CYCLONEDX_VERSION.crate

CYCLONEDX_CLI_VERSION=0.33.1
CYCLONEDX_CLI_SHA256=bfc8b2538da86fe239bc53658bbb63c1c8c510a293c1e6891aa5bea5d3c58746
CYCLONEDX_CLI_URL=https://github.com/CycloneDX/cyclonedx-cli/releases/download/v$CYCLONEDX_CLI_VERSION/cyclonedx-linux-x64

# Install cargo-cyclonedx
curl --proto '=https' --tlsv1.2 -sSfL -o /tmp/cargo-cyclonedx.crate "$CARGO_CYCLONEDX_URL"
echo "$CARGO_CYCLONEDX_SHA256  /tmp/cargo-cyclonedx.crate" | sha256sum -c -
tar -xzf /tmp/cargo-cyclonedx.crate -C /tmp
cargo install --locked --path "/tmp/cargo-cyclonedx-$CARGO_CYCLONEDX_VERSION"
rm -r /tmp/cargo-cyclonedx.crate "/tmp/cargo-cyclonedx-$CARGO_CYCLONEDX_VERSION"

# Install cyclonedx-cli
curl --proto '=https' --tlsv1.2 -sSfL -o /usr/local/bin/cyclonedx "$CYCLONEDX_CLI_URL"
echo "$CYCLONEDX_CLI_SHA256  /usr/local/bin/cyclonedx" | sha256sum -c -
chmod +x /usr/local/bin/cyclonedx
