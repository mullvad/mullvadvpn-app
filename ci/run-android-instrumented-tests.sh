#!/usr/bin/env bash

adb install ./debug/app-debug.apk
adb install ./androidTest/debug/app-debug-androidTest.apk
adb shell am instrument -w net.mullvad.mullvadvpn.test/androidx.test.runner.AndroidJUnitRunner
