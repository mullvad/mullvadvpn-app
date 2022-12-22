#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "### Updating dependency lockfile ###"
echo ""

echo "Removing old components..."
sed -i '/<components>/,/<\/components>/d' ../gradle/verification-metadata.xml

echo "Generating new components..."
android_container_image_name=$(cat "../../building/android-container-image.txt")
podman run --rm -it \
    -v ../..:/build:Z \
    "$android_container_image_name" \
    android/gradlew -q -p android -M sha256 assemble assembleAndroidTest
