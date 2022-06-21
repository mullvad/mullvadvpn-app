#!/usr/bin/env bash

set -eux

export RUSTFLAGS="--deny warnings"

# Check rust crates with clippy
source env.sh
time cargo clippy --locked --verbose
