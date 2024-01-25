#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cd "$SCRIPT_DIR"/..
./gradlew assembleOssProdAndroidTest
./gradlew app:assembleOssProdDebug
"$SCRIPT_DIR"/run-instrumented-tests.sh app
