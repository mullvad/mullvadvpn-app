#!/usr/bin/env bash

APK_BASE_DIR=$1
adb install "$APK_BASE_DIR/debug/app-debug.apk"
adb install "$APK_BASE_DIR/androidTest/debug/app-debug-androidTest.apk"
adb shell am instrument -w net.mullvad.mullvadvpn.test/androidx.test.runner.AndroidJUnitRunner
