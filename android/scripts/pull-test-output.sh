#!/usr/bin/env bash

set -eu

# Must match the path where e2e tests output their attachments
TEST_DEVICE_OUTPUTS_DIR="${TEST_DEVICE_OUTPUTS_DIR:-/sdcard/Download/test-outputs}"
REPORT_DIR="${REPORT_DIR:-}"

if [[ -z $TEST_DEVICE_OUTPUTS_DIR ]]; then
    echo ""
    echo "Error: The variable TEST_DEVICE_OUTPUTS_DIR must be set."
    exit 1
fi

if [[ -z $REPORT_DIR || ! -d $REPORT_DIR ]]; then
    echo ""
    echo "Error: The variable REPORT_DIR must be set and the directory must exist."
    exit 1
fi

echo "Collecting produced test attachments and logs..."
adb pull "$TEST_DEVICE_OUTPUTS_DIR" "$REPORT_DIR"
