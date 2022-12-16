#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "### Updating dependency lockfile ###"
echo ""

echo "Removing old components..."
sed -i '/<components>/,/<\/components>/d' ../gradle/verification-metadata.xml

echo "Generating new components..."
podman run --rm -it \
    -v ../..:/build:Z \
    ghcr.io/mullvad/mullvadvpn-app-build-android:06d988a5a \
    android/gradlew -q -p android -M sha256 assembleAndroidTest
