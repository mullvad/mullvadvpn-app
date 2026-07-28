#!/usr/bin/env bash

set -eux

# Hard deny on all warnings when running in CI. Covers rustc, clippy and rustdoc
# lints in local packages.
export CARGO_BUILD_WARNINGS=deny

# Allow private-intra-doc-links, since they are still useful in editor,
# and we're not publishing these crates on docs.rs anyway.
export RUSTDOCFLAGS="--allow rustdoc::private-intra-doc-links"

exec cargo --locked --color=always "$@"
