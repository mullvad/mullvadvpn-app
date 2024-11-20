#!/usr/bin/env bash

set -eu

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

if [[ -z $REPORT_DIR || ! -d $REPORT_DIR ]]; then
    echo ""
    echo "Error: The variable REPORT_DIR must be set and the directory must exist."
    exit 1
fi

DEVICE_SCREENSHOT_PATH="/sdcard/Pictures/mullvad-$TEST_TYPE"
DEVICE_TEST_ATTACHMENTS_PATH="/sdcard/Download/test-attachments"
LOCAL_LOGCAT_FILE_PATH="$REPORT_DIR/logcat.txt"
LOCAL_SCREENSHOT_PATH="$REPORT_DIR/screenshots"
LOCAL_TEST_ARRACHMENTS_PATH="$REPORT_DIR/test-attachments"

echo "Collecting report and produced test attachments..."
adb logcat -d > "$LOCAL_LOGCAT_FILE_PATH"
adb pull "$DEVICE_SCREENSHOT_PATH" "$LOCAL_SCREENSHOT_PATH" 2>&1 || echo "No screenshots"
adb pull "$DEVICE_TEST_ATTACHMENTS_PATH" "$LOCAL_TEST_ARRACHMENTS_PATH" 2>&1 || echo "No test attachments"
