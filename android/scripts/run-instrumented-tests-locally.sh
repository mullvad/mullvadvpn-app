#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cd "$SCRIPT_DIR"/..
./gradlew assembleOssProdAndroidTest
./gradlew app:assembleOssProdDebug
export REPORT_DIR=$(mktemp -d)
echo REPORTDIR
echo $REPORT_DIR
"$SCRIPT_DIR"/run-instrumented-tests.sh --test-type app --infra-flavor prod --billing-flavor oss

