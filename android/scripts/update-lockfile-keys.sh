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
    "assemble"
    "compileDebugUnitTestKotlin"
    "assembleAndroidTest"
    "lint"
)
EXCLUDED_GRADLE_TASKS=(
    "-xensureRelayListExist"
    "-xensureJniDirectoryExist"
)

export GRADLE_OPTS
export GRADLE_USER_HOME

function cleanup {
    echo "Cleaning up temp dirs..."
    rm -rf -- "$TEMP_GRADLE_PROJECT_CACHE_DIR" ../gradle/verification-metadata.dryrun.xml ../gradle/verification-keyring.dryrun.keys ../gradle/verification-keyring.dryrun.gpg
}

trap cleanup EXIT

echo "### Updating dependency lockfile verification keys ###"
echo ""

echo "Set key servers true temporarily"
sed -Ei 's,key-servers enabled="[^"]+",key-servers enabled="true",' ../gradle/verification-metadata.xml

# Generate keys

echo "Generating new trusted keys..."
# Using a loop here since providing all tasks at once result in gradle task dependency issues.
for GRADLE_TASK in "${GRADLE_TASKS[@]}"; do
    echo "Gradle task: $GRADLE_TASK"
    ../gradlew -q -p .. --project-cache-dir "$TEMP_GRADLE_PROJECT_CACHE_DIR" -M pgp,sha256 "$GRADLE_TASK" --export-keys --dry-run "${EXCLUDED_GRADLE_TASKS[@]}"
    echo ""
done

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

echo "sorting keyring and removing duplicates"
  # sort and unique the keyring
  # https://github.com/gradle/gradle/issues/20140
  # `sed 's/$/NEWLINE/g'` adds the word NEWLINE at the end of each line
  # `tr -d '\n'` deletes the actual newlines
  # `sed` again adds a newline at the end of each key, so each key is one line
  # `sort` orders the keys deterministically
  # `uniq` removes identical keys
  # `sed 's/NEWLINE/\n/g'` puts the newlines back
cat ../gradle/verification-keyring.dryrun.keys \
    | sed 's/$/NEWLINE/g' \
    | tr -d '\n' \
    | sed 's/\(-----END PGP PUBLIC KEY BLOCK-----\)/\1\n/g' \
    | grep "END PGP PUBLIC KEY BLOCK" \
    | sort \
    | uniq \
    | sed 's/NEWLINE/\n/g' \
    > ../gradle/verification-keyring.keys

echo "Disable key servers again"
sed -Ei 's,key-servers enabled="[^"]+",key-servers enabled="false",' ../gradle/verification-metadata.xml

