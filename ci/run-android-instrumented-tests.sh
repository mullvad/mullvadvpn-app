#!/usr/bin/env bash

APK_BASE_DIR=$1
LOG_FILE_NAME="mullvad.instrumentation_data_proto"
LOG_FILE_PATH="/sdcard/$LOG_FILE_NAME"
LOG_FAILURE_MESSAGE="FAILURES\!\!\!"

echo "Running instrumented tests..."
echo ""

echo "### Ensure uninstalled ###"
adb uninstall net.mullvad.mullvadvpn || echo "App package not installed"
adb uninstall net.mullvad.mullvadvpn.test || echo "Test package not installed"
echo ""

echo "### Install ###"
adb install -t "$APK_BASE_DIR/debug/app-debug.apk"
adb install "$APK_BASE_DIR/androidTest/debug/app-debug-androidTest.apk"
echo ""

echo "### Start tests ###"
adb shell am instrument -w -f "$LOG_FILE_NAME" net.mullvad.mullvadvpn.test/androidx.test.runner.AndroidJUnitRunner
echo ""

# Print log so that it shows as part of GitHub Actions logs etc
echo "### Logs ###"
adb shell cat "$LOG_FILE_PATH"

if adb shell grep -q "$LOG_FAILURE_MESSAGE" "$LOG_FILE_PATH"; then
    exit 1
fi
