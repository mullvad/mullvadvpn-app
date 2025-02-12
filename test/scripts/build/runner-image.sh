#!/usr/bin/env bash

# This script produces a virtual disk containing the test runner binaries.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TEST_FRAMEWORK_ROOT="$SCRIPT_DIR/../.."

TEST_RUNNER_IMAGE_SIZE_MB=5000

case $TARGET in
    "x86_64-pc-windows-gnu")
        TEST_RUNNER_IMAGE_FILENAME=windows-test-runner.img
        ;;
    *)
        echo "Unknown target: $TARGET"
        exit 1
        ;;
esac

echo "************************************************************"
echo "* Preparing test runner image: $TARGET"
echo "************************************************************"

mkdir -p "${TEST_FRAMEWORK_REPO}/testrunner-images"
TEST_RUNNER_IMAGE_PATH="${TEST_FRAMEWORK_REPO}/testrunner-images/${TEST_RUNNER_IMAGE_FILENAME}"

case $TARGET in

    "x86_64-pc-windows-gnu")
        truncate -s "${TEST_RUNNER_IMAGE_SIZE_MB}M" "${TEST_RUNNER_IMAGE_PATH}"
        mformat -F -i "${TEST_RUNNER_IMAGE_PATH}" "::"
        mcopy \
            -i "${TEST_RUNNER_IMAGE_PATH}" \
            "${TEST_FRAMEWORK_ROOT}/target/$TARGET/release/test-runner.exe" \
            "${TEST_FRAMEWORK_ROOT}/target/$TARGET/release/connection-checker.exe" \
            "${PACKAGE_DIR}/"*.exe \
            "::"
        mdir -i "${TEST_RUNNER_IMAGE_PATH}"
        ;;

esac

echo "************************************************************"
echo "* Success! Built test runner image: $TARGET"
echo "************************************************************"
