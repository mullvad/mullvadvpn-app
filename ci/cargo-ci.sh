#!/usr/bin/env bash

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Explicitly pass empty first argument, otherwise this script will pass on its arguments in a `source` invocation.
# shellcheck disable=SC1091
source "$SCRIPT_DIR/../env.sh" ""

# Hard deny on all warnings when running in CI
export RUSTFLAGS="--deny warnings"

exec cargo --locked "$@"
