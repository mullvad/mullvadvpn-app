#!/usr/bin/env bash

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Explicitly pass empty first argument, otherwise this script will pass on its arguments in a `source` invocation.
# shellcheck disable=SC1091
source "$SCRIPT_DIR/../env.sh" ""

# Hard deny on all warnings when running in CI
export RUSTFLAGS="--deny warnings"

# Deny broken docstrings when running in CI
# Allow private-intra-doc-links, since they are still useful in editor,
# and we're not publishing these crates on docs.rs anyway.
export RUSTDOCFLAGS="--deny warnings --allow rustdoc::private-intra-doc-links"

exec cargo --locked "$@"
