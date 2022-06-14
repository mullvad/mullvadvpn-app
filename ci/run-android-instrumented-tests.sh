#!/usr/bin/env bash

APK_BASE_DIR=$1
LOG_FILE_NAME="mullvad.instrumentation_data_proto"
LOG_FILE_PATH="/sdcard/$LOG_FILE_NAME"
LOG_FAILURE_MESSAGE="FAILURES\!\!\!"

adb install -r -t "$APK_BASE_DIR/debug/app-debug.apk"
adb install "$APK_BASE_DIR/androidTest/debug/app-debug-androidTest.apk"
adb shell am instrument -w -f "$LOG_FILE_NAME" net.mullvad.mullvadvpn.test/androidx.test.runner.AndroidJUnitRunner

# Print log so that it shows as part of GitHub Actions logs etc
adb shell cat "$LOG_FILE_PATH"

if adb shell grep -q "$LOG_FAILURE_MESSAGE" "$LOG_FILE_PATH"; then
    exit 1
fi
