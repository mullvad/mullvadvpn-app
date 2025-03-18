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
# Task list to discover all tasks and their dependencies since
# just running the suggested 'help' task isn't sufficient.
GRADLE_TASKS=(
    "lint"
)

export GRADLE_OPTS
export GRADLE_USER_HOME

cd ../gradle/

function cleanup {
    echo "Cleaning up temp dirs..."
    rm -rf -- "$GRADLE_USER_HOME" "$TEMP_GRADLE_PROJECT_CACHE_DIR"
}

trap cleanup EXIT

echo "### Configuration ###"
echo "Gradle home: $GRADLE_USER_HOME"
echo "Gradle cache: $TEMP_GRADLE_PROJECT_CACHE_DIR"
echo ""

echo "### Verifying packages ###"
echo "Moving checksums to the side..."
mv verification-metadata.xml verification-metadata.checksums.xml

echo "Moving keys to be active metadata file"
mv verification-metadata.keys.xml verification-metadata.xml

echo "Generating new components..."
# Using a loop here since providing all tasks at once result in gradle task dependency issues.
for GRADLE_TASK in "${GRADLE_TASKS[@]}"; do
    echo "Gradle task: $GRADLE_TASK"
    ../gradlew -q -p .. --project-cache-dir "$TEMP_GRADLE_PROJECT_CACHE_DIR" "$GRADLE_TASK"
    echo ""
done

echo "Moving back keys verification metadata"
mv verification-metadata.xml verification-metadata.keys.xml

echo ""
echo "Moving back checksums to be active metadata file"
mv verification-metadata.checksums.xml verification-metadata.xml
