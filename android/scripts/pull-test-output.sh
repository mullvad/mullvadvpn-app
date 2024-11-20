#!/usr/bin/env bash

set -eu

TEST_DEVICE_OUTPUTS_DIR="${TEST_DEVICE_OUTPUTS_DIR:-/sdcard/Download/test-outputs/attachments}" # Must match the path where e2e tests output their attachments
REPORT_DIR="${REPORT_DIR:-}"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --test-type)
            if [[ -n "${2-}" && "$2" =~ ^(app|mockapi|e2e)$ ]]; then
                TEST_TYPE="$2"
            else
                echo "Error: Bad or missing test type. Must be one of: app, mockapi, e2e"
                exit 1
            fi
            shift 2
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

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
