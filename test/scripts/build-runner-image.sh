#!/usr/bin/env bash

# This script produces a virtual disk containing the test runner binaries.

set -eu

TEST_RUNNER_IMAGE_SIZE_MB=1000

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

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

mkdir -p "${SCRIPT_DIR}/../testrunner-images/"
TEST_RUNNER_IMAGE_PATH="${SCRIPT_DIR}/../testrunner-images/${TEST_RUNNER_IMAGE_FILENAME}"

case $TARGET in

    "x86_64-pc-windows-gnu")
        truncate -s "${TEST_RUNNER_IMAGE_SIZE_MB}M" "${TEST_RUNNER_IMAGE_PATH}"
        mformat -F -i "${TEST_RUNNER_IMAGE_PATH}" "::"
        mcopy \
            -i "${TEST_RUNNER_IMAGE_PATH}" \
            "${SCRIPT_DIR}/../target/$TARGET/release/test-runner.exe" \
            "${PACKAGES_DIR}/"*.exe \
            "${SCRIPT_DIR}/../openvpn.ca.crt" \
            "::"
        mdir -i "${TEST_RUNNER_IMAGE_PATH}"
        ;;

esac

echo "************************************************************"
echo "* Success! Built test runner image: $TARGET"
echo "************************************************************"
