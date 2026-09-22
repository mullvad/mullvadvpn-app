#!/usr/bin/env bash

set -eux

# Hard deny on all warnings when running in CI. Covers rustc, clippy and rustdoc
# lints in local packages.
export CARGO_BUILD_WARNINGS=deny

# Allow private-intra-doc-links, since they are still useful in editor,
# and we're not publishing these crates on docs.rs anyway.
export RUSTDOCFLAGS="--allow rustdoc::private-intra-doc-links"

# Print a backtrace when a test panics. The `ci` profile keeps line tables, so
# the frames carry file names and line numbers.
export RUST_BACKTRACE=1

# Build with the `ci` profile, defined in the `Cargo.toml` of both workspaces.
# `--profile` is a subcommand flag, so it has to go after the subcommand.
subcommand="$1"
shift

exec cargo --locked --color=always "$subcommand" --profile ci "$@"
