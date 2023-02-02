#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

TEST_TYPE=$1
REPEAT_COUNT=$2

for ((i=1; i <= REPEAT_COUNT; i++))
do
    echo "### Run $i ###"
    ./run-instrumented-tests.sh "$TEST_TYPE"
done
