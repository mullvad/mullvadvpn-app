#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Disable daemon since it causes problems with the temp dir cleanup
# regardless if stopped.
GRADLE_OPTS="-Dorg.gradle.daemon=false"
# We must provide a template for mktemp to work properly on macOS.
TEMP_GRADLE_PROJECT_CACHE_DIR=$(mktemp -d -t gradle-cache-XXX)
EXCLUDED_GRADLE_TASKS=(
    "-xensureRelayListExist"
    "-xensureJniDirectoryExist"
)

function cleanup {
    echo "Cleaning up temp dirs..."
    rm -rf -- "$TEMP_GRADLE_PROJECT_CACHE_DIR" ../gradle/verification-metadata.dryrun.xml
}

trap cleanup EXIT

echo "### Updating dependency lockfile verification keys ###"
echo ""

# Generate keys

echo "Generating new trusted keys..."
../gradlew -q -p .. --project-cache-dir "$TEMP_GRADLE_PROJECT_CACHE_DIR" -M pgp,sha256 help --dry-run "${EXCLUDED_GRADLE_TASKS[@]}"
echo ""

# Move keys from dry run file to existing file (This part is taken from: https://gitlab.com/fdroid/fdroidclient/-/blob/master/gradle/update-verification-metadata.sh)
# extract the middle of the new file, https://github.com/gradle/gradle/issues/18569
grep -B 10000 "<trusted-keys>" ../gradle/verification-metadata.dryrun.xml > "$TEMP_GRADLE_PROJECT_CACHE_DIR/new.head"
grep -A 10000 "</trusted-keys>" ../gradle/verification-metadata.dryrun.xml > "$TEMP_GRADLE_PROJECT_CACHE_DIR/new.tail"
numTopLines="$(cat "$TEMP_GRADLE_PROJECT_CACHE_DIR/new.head" | wc -l)"
numTopLinesPlus1="$(($numTopLines + 1))"
numBottomLines="$(cat "$TEMP_GRADLE_PROJECT_CACHE_DIR/new.tail" | wc -l)"
numLines="$(cat ../gradle/verification-metadata.dryrun.xml | wc -l)"
numMiddleLines="$(($numLines - $numTopLines - $numBottomLines))"
# also remove 'version=' lines, https://github.com/gradle/gradle/issues/20192
cat ../gradle/verification-metadata.dryrun.xml | tail -n "+$numTopLinesPlus1" | head -n "$numMiddleLines" | sed 's/ version="[^"]*"//' > "$TEMP_GRADLE_PROJECT_CACHE_DIR/new.middle"

# extract the top and bottom of the old file
grep -B 10000 "<trusted-keys>" ../gradle/verification-metadata.xml > "$TEMP_GRADLE_PROJECT_CACHE_DIR/old.head"
grep -A 10000 "</trusted-keys>" ../gradle/verification-metadata.xml > "$TEMP_GRADLE_PROJECT_CACHE_DIR/old.tail"

# update verification metadata file
cat "$TEMP_GRADLE_PROJECT_CACHE_DIR/old.head" "$TEMP_GRADLE_PROJECT_CACHE_DIR/new.middle" "$TEMP_GRADLE_PROJECT_CACHE_DIR/old.tail" > ../gradle/verification-metadata.xml

