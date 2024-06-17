#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

GRADLE_TASKS=("assemble" "compileDebugUnitTestKotlin" "assembleAndroidTest" "lint")

TEMP_GRADLE_HOME_DIR=$(mktemp -d)
TEMP_GRADLE_PROJECT_CACHE_DIR=$(mktemp -d)

function cleanup {
    echo "Cleaning up temp dirs..."
    rm -rf -- "$TEMP_GRADLE_HOME_DIR" "$TEMP_GRADLE_PROJECT_CACHE_DIR"
}

trap cleanup EXIT

echo "### Updating dependency lockfile ###"
echo ""

echo "Removing old components..."
sed -i '/<components>/,/<\/components>/d' ../gradle/verification-metadata.xml

echo "Generating new components..."
../gradlew -q -p .. -g "$TEMP_GRADLE_HOME_DIR" --project-cache-dir "$TEMP_GRADLE_PROJECT_CACHE_DIR" -M sha256 "${GRADLE_TASKS[@]}"
