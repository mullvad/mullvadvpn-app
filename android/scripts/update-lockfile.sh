#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Disable daemon since it causes problems with the temp dir cleanup
# regardless if stopped.
GRADLE_OPTS="-Dorg.gradle.daemon=false"
# We must provide a template for mktemp to work properly on macOS.
GRADLE_USER_HOME=$(mktemp -d -t gradle-home-XXX)
TEMP_GRADLE_PROJECT_CACHE_DIR=$(mktemp -d -t gradle-cache-XXX)
GRADLE_TASKS=(
    "assemble"
    "compileDebugUnitTestKotlin"
    "assembleAndroidTest"
    "lint"
    "-x:app:ensureRelayListExist"
    "-x:app:ensureJniDirectoryExist"
)

export GRADLE_OPTS
export GRADLE_USER_HOME

function cleanup {
    echo "Cleaning up temp dirs..."
    rm -rf -- "$GRADLE_USER_HOME" "$TEMP_GRADLE_PROJECT_CACHE_DIR"
}

trap cleanup EXIT

echo "### Updating dependency lockfile ###"
echo "Gradle home: $GRADLE_USER_HOME"
echo "Gradle cache: $TEMP_GRADLE_PROJECT_CACHE_DIR"
echo ""

echo "Removing old components..."
sed -i '/<components>/,/<\/components>/d' ../gradle/verification-metadata.xml

echo "Generating new components..."
../gradlew -q -p .. --project-cache-dir "$TEMP_GRADLE_PROJECT_CACHE_DIR" -M sha256 "${GRADLE_TASKS[@]}"
