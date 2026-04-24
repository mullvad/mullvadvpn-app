#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

printf "\nWARNING: This script is deprecated and only kept as a build server \
compatibility wrapper. It disregards any arguments and simply runs: \
./gradlew --console plain fullRelease\n\n"

./gradlew --console plain fullRelease
