#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "### Updating dependency lockfile ###"
echo ""

echo "Removing old components..."
sed -i '/<components>/,/<\/components>/d' ../gradle/verification-metadata.xml

echo "Generating new components..."
../gradlew -q -p .. -M sha256 assemble compileDebugUnitTestKotlin assembleAndroidTest lint
