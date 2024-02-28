#!/usr/bin/env bash

# Help script to invoke run-instrumented-tests.sh multiple times.
#
# Usage:
# run-instrumented-tests-repeat.sh <repeat-count> [<args>]
#
# Example:
# run-instrumented-tests-repeat.sh 2 --test-type mockapi --infra-flavor prod --billing-flavor oss

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

REPEAT_COUNT=$1

for ((i=1; i <= REPEAT_COUNT; i++))
do
    echo "### Run $i ###"
    ./run-instrumented-tests.sh "${@:2}"
done
