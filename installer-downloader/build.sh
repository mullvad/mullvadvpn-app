#!/usr/bin/env bash

# This script is used to build, and optionally sign the downloader, always in release mode.

# This script performs the equivalent of the following profile:
#
# [profile.release]
# strip = true
# opt-level = 'z'
# codegen-units = 1
# lto = true
# panic = 'abort'
#
# We cannot set all of the above directly in Cargo.toml since some must be set for the entire
# workspace.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# shellcheck disable=SC1091
source ../scripts/utils/host
# shellcheck disable=SC1091
source ../scripts/utils/log

RUSTFLAGS="-C codegen-units=1 -C panic=abort -C strip=symbols -C opt-level=z" \
    cargo build --bin installer-downloader --release
