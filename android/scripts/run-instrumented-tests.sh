#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

APK_BASE_DIR=${APK_BASE_DIR:-"$SCRIPT_DIR/.."}
LOG_FAILURE_MESSAGE="FAILURES!!!"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        app)
            TEST_TYPE="app"
            USE_ORCHESTRATOR="false"
            TEST_PACKAGE="net.mullvad.mullvadvpn.test"
            TEST_APK="$APK_BASE_DIR/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk"
            ;;
        e2e)
            TEST_TYPE="e2e"
            USE_ORCHESTRATOR="true"
            TEST_PACKAGE="net.mullvad.mullvadvpn.test.$TEST_TYPE"
            TEST_APK="$APK_BASE_DIR/test/$TEST_TYPE/build/outputs/apk/debug/$TEST_TYPE-debug.apk"
            if [[ -z ${VALID_TEST_ACCOUNT_TOKEN-} ]]; then
                echo "The variable VALID_TEST_ACCOUNT_TOKEN is not set."
                exit 1
            fi
            if [[ -z ${INVALID_TEST_ACCOUNT_TOKEN-} ]]; then
                echo "The variable INVALID_TEST_ACCOUNT_TOKEN is not set."
                exit 1
            fi
            OPTIONAL_TEST_ARGUMENTS="\
            -e valid_test_account_token $VALID_TEST_ACCOUNT_TOKEN \
            -e invalid_test_account_token $INVALID_TEST_ACCOUNT_TOKEN"
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
    shift
done

if [[ -z ${TEST_TYPE-} ]]; then
    echo "Missing test type argument. Should be one of: app, e2e"
    exit 1
fi

if [[ "${ORCHESTRATOR_APK_PATH-}" == "true" ]]; then
    if [[ -z ${ORCHESTRATOR_APK_PATH-} ]]; then
        echo "The variable ORCHESTRATOR_APK_PATH is not set."
        exit 1
    fi
    if [[ -z ${TEST_SERVICES_APK_PATH-} ]]; then
        echo "The variable TEST_SERVICES_APK_PATH is not set."
        exit 1
    fi
fi

LOG_FILE_NAME="mullvad-$TEST_TYPE.txt"
LOG_FILE_PATH="/tmp/$LOG_FILE_NAME"

echo "Starting instrumented tests of type: $TEST_TYPE"
echo ""

echo "### Ensure that packages are not previously installed ###"
adb uninstall net.mullvad.mullvadvpn || echo "App package not installed"
adb uninstall "$TEST_PACKAGE" || echo "Test package not installed"
adb uninstall androidx.test.services || echo "Test services package not installed"
adb uninstall androidx.test.orchestrator || echo "Test orchestrator package not installed"
echo ""

echo "### Install packages ###"
adb install -t "$APK_BASE_DIR/app/build/outputs/apk/debug/app-debug.apk"
adb install "$TEST_APK"
if [[ "$USE_ORCHESTRATOR" == "true" ]]; then
    adb install "$ORCHESTRATOR_APK_PATH"
    adb install "$TEST_SERVICES_APK_PATH"
fi
echo ""

echo "### Run instrumented test command ###"
if [[ "$USE_ORCHESTRATOR" == "true" ]]; then
    INSTRUMENTATION_COMMAND="\
    CLASSPATH=\$(pm path androidx.test.services) app_process / androidx.test.services.shellexecutor.ShellMain \
    am instrument -r -w \
    -e targetInstrumentation $TEST_PACKAGE/androidx.test.runner.AndroidJUnitRunner \
    -e clearPackageData true \
    ${OPTIONAL_TEST_ARGUMENTS:-""} \
    androidx.test.orchestrator/androidx.test.orchestrator.AndroidTestOrchestrator"
else
    INSTRUMENTATION_COMMAND="\
    am instrument -w \
    $TEST_PACKAGE/androidx.test.runner.AndroidJUnitRunner"
fi
adb shell "$INSTRUMENTATION_COMMAND" | tee "$LOG_FILE_PATH"
echo ""

echo "### Ensure that packages are uninstalled ###"
adb uninstall net.mullvad.mullvadvpn || echo "App package not installed"
adb uninstall "$TEST_PACKAGE" || echo "Test package not installed"
adb uninstall androidx.test.services || echo "Test services package not installed"
adb uninstall androidx.test.orchestrator || echo "Test orchestrator package not installed"
echo ""

echo "### Checking logs for failures ###"
if grep -q "$LOG_FAILURE_MESSAGE" "$LOG_FILE_PATH"; then
    echo "One or more tests failed, see logs for more details."
    exit 1
else
    echo "No failures!"
fi
