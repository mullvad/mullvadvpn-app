#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

refresh_all_keys_flag=false

print_usage() {
  echo "Usage:"
  echo "    -r        Refresh all keys, will remove all trusted keys and clear the keyring, allowing for old keys to removed and keys entries to be updated."
  echo "              This result is not reproducible. Also make sure to do an additional normal run afterwards."
  echo "    -h        Show this help page."
}

while getopts 'rh' flag; do
  case "${flag}" in
    r) refresh_all_keys_flag=true ;;
    *) print_usage
       exit 1 ;;
  esac
done

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
    rm -rf -- "$GRADLE_USER_HOME" "$TEMP_GRADLE_PROJECT_CACHE_DIR" verification-keyring.gpg
}

trap cleanup EXIT

echo "### Configuration ###"
echo "Gradle home: $GRADLE_USER_HOME"
echo "Gradle cache: $TEMP_GRADLE_PROJECT_CACHE_DIR"
echo ""

echo "### Updating checksums ###"
echo ""

echo "Removing old components..."
sed -i '/<components>/,/<\/components>/d' verification-metadata.xml
echo ""

echo "Generating new components..."
# Using a loop here since providing all tasks at once result in gradle task dependency issues.
for GRADLE_TASK in "${GRADLE_TASKS[@]}"; do
    echo "Gradle task: $GRADLE_TASK"
    ../gradlew -q -p .. --project-cache-dir "$TEMP_GRADLE_PROJECT_CACHE_DIR" -M sha256 "$GRADLE_TASK"
    echo ""
done

echo "Moving checksums to the side..."
mv verification-metadata.xml verification-metadata.checksums.xml



echo "### Updating keys metadata ###"
echo ""

echo "Moving keys to be active metadata file"
mv verification-metadata.keys.xml verification-metadata.xml


echo "Temporarily enabling key servers..."
sed -Ei 's,key-servers enabled="[^"]+",key-servers enabled="true",' verification-metadata.xml

echo "Removing old components..."
sed -i '/<components>/,/<\/components>/d' verification-metadata.xml
echo ""


if [ "$refresh_all_keys_flag" = true ]; then
    echo "Refreshing all keys"

    echo "Removing old trusted keys..."
    sed -i '/<trusted-keys>/,/<\/trusted-keys>/d' verification-metadata.xml
    echo ""

    echo "Removing old keyring..."
    rm verification-keyring.keys
    echo ""
fi

echo "Generating new trusted keys & updating keyring..."
../gradlew -q -p .. --project-cache-dir "$TEMP_GRADLE_PROJECT_CACHE_DIR" -M pgp,sha256 "${GRADLE_TASKS[@]}" --export-keys

echo "Sorting keyring and removing duplicates..."
  # Sort and unique the keyring
  # https://github.com/gradle/gradle/issues/20140
  # `sed 's/$/NEWLINE/g'` adds the word NEWLINE at the end of each line
  # `tr -d '\n'` deletes the actual newlines
  # `sed` again adds a newline at the end of each key, so each key is one line
  # `sort` orders the keys deterministically
  # `uniq` removes identical keys
  # `sed 's/NEWLINE/\n/g'` puts the newlines back
< verification-keyring.keys \
    sed 's/$/NEWLINE/g' \
    | tr -d '\n' \
    | sed 's/\(-----END PGP PUBLIC KEY BLOCK-----\)/\1\n/g' \
    | grep "END PGP PUBLIC KEY BLOCK" \
    | sort \
    | uniq \
    | sed 's/NEWLINE/\n/g' \
    > verification-keyring.new.keys

mv -f verification-keyring.new.keys verification-keyring.keys

echo "Disabling key servers..."
sed -Ezi 's,key-servers,key-servers enabled="false",' verification-metadata.xml

echo "Moving back keys verification metadata"
mv verification-metadata.xml verification-metadata.keys.xml

echo ""
echo "Moving checksums to be active metadata file"
mv verification-metadata.checksums.xml verification-metadata.xml
