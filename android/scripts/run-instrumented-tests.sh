#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

AUTO_FETCH_TEST_HELPER_APKS=${AUTO_FETCH_TEST_HELPER_APKS:-"false"}

APK_BASE_DIR=${APK_BASE_DIR:-"$SCRIPT_DIR/.."}
LOG_FAILURE_MESSAGE="FAILURES!!!"

DEFAULT_ORCHESTRATOR_APK_PATH=/tmp/orchestrator.apk
DEFAULT_TEST_SERVICES_APK_PATH=/tmp/test-services.apk

ORCHESTRATOR_URL=https://dl.google.com/android/maven2/androidx/test/orchestrator/1.4.2/orchestrator-1.4.2.apk
TEST_SERVICES_URL=https://dl.google.com/android/maven2/androidx/test/services/test-services/1.4.2/test-services-1.4.2.apk

PARTNER_AUTH="${PARTNER_AUTH:-}"
VALID_TEST_ACCOUNT_TOKEN="${VALID_TEST_ACCOUNT_TOKEN:-}"
INVALID_TEST_ACCOUNT_TOKEN="${INVALID_TEST_ACCOUNT_TOKEN:-}"

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
        --infra-flavor)
            if [[ -n "${2-}" && "$2" =~ ^(prod|stagemole)$ ]]; then
                INFRA_FLAVOR="$2"
            else
                echo "Error: Bad or missing infra flavor. Must be one of: prod, stagemole"
                exit 1
            fi
            shift 2
            ;;
        --billing-flavor)
            if [[ -n "${2-}" && "$2" =~ ^(oss|play)$ ]]; then
                BILLING_FLAVOR="$2"
            else
                echo "Error: Bad or missing billing flavor. Must be one of: oss, play"
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

if [[ -z ${TEST_TYPE-} ]]; then
    echo "Error: Missing --test-type argument. Must be set to one of: app, e2e, mockapi"
    exit 1
fi

if [[ -z ${INFRA_FLAVOR-} ]]; then
    echo "Error: Missing --infra-flavor argument. Must be set to one of: prod, stagemole"
    exit 1
fi

if [[ -z ${BILLING_FLAVOR-} ]]; then
    echo "Error: Missing --billing-flavor argument. Must be set to one of: oss, play"
    exit 1
fi

echo "### Configuration ###"
echo "Test type: $TEST_TYPE"
echo "Infra flavor: $INFRA_FLAVOR"
echo "Billing flavor: $BILLING_FLAVOR"

APK_PATH="$APK_BASE_DIR/app/build/outputs/apk/$BILLING_FLAVOR${INFRA_FLAVOR^}/debug/app-$BILLING_FLAVOR-$INFRA_FLAVOR-debug.apk"

case "$TEST_TYPE" in
    app)
    if [[ $BILLING_FLAVOR != "oss" || $INFRA_FLAVOR != "prod" ]]; then
        echo ""
        echo "Error: The 'app' test type only supports billing type 'oss' and infra type 'prod'."
        exit 1
    fi
    USE_ORCHESTRATOR="false"
    PACKAGE_NAME="net.mullvad.mullvadvpn"
    TEST_PACKAGE_NAME="net.mullvad.mullvadvpn.test"
    TEST_APK_PATH="$APK_BASE_DIR/app/build/outputs/apk/androidTest/$BILLING_FLAVOR${INFRA_FLAVOR^}/debug/app-$BILLING_FLAVOR-$INFRA_FLAVOR-debug-androidTest.apk"
    ;;
    mockapi)

    if [[ $BILLING_FLAVOR != "oss" || $INFRA_FLAVOR != "prod" ]]; then
        echo ""
        echo "Error: The 'mockapi' test type only supports billing type 'oss' and infra type 'prod'."
        exit 1
    fi
    USE_ORCHESTRATOR="true"
    PACKAGE_NAME="net.mullvad.mullvadvpn"
    TEST_PACKAGE_NAME="net.mullvad.mullvadvpn.test.mockapi"
    TEST_APK_PATH="$APK_BASE_DIR/test/mockapi/build/outputs/apk/$BILLING_FLAVOR/debug/mockapi-$BILLING_FLAVOR-debug.apk"
    ;;

    e2e)
    if [[ $BILLING_FLAVOR == "play" && $INFRA_FLAVOR != "stagemole" ]]; then
        echo ""
        echo "Error: The 'e2e' test type with billing flavor 'play' require infra flavor 'stagemole'."
        exit 1
    elif [[ $BILLING_FLAVOR == "oss" && $INFRA_FLAVOR != "prod" ]]; then
        echo ""
        echo "Error: The 'e2e' test type with billing flavor 'oss' require infra flavor 'prod'."
        exit 1
    fi
    OPTIONAL_TEST_ARGUMENTS=""
    if [[ -n ${INVALID_TEST_ACCOUNT_TOKEN-} ]]; then
        OPTIONAL_TEST_ARGUMENTS+=" -e invalid_test_account_token $INVALID_TEST_ACCOUNT_TOKEN"
    else
        echo "Error: The variable INVALID_TEST_ACCOUNT_TOKEN must be set."
        exit 1
    fi
    if [[ -n ${PARTNER_AUTH} ]]; then
        echo "Test account used for e2e test (provided/partner): partner"
        OPTIONAL_TEST_ARGUMENTS+=" -e partner_auth $PARTNER_AUTH"
    elif [[ -n ${VALID_TEST_ACCOUNT_TOKEN} ]]; then
        echo "Test account used for e2e test (provided/partner): provided"
        OPTIONAL_TEST_ARGUMENTS+=" -e valid_test_account_token $VALID_TEST_ACCOUNT_TOKEN"
    else
        echo ""
        echo "Error: The variable PARTNER_AUTH or VALID_TEST_ACCOUNT_TOKEN must be set."
        exit 1
    fi
    USE_ORCHESTRATOR="true"
    PACKAGE_NAME="net.mullvad.mullvadvpn"
    if [[ "$INFRA_FLAVOR" =~ ^(devmole|stagemole)$ ]]; then
        PACKAGE_NAME+=".$INFRA_FLAVOR"
    fi
    TEST_PACKAGE_NAME="net.mullvad.mullvadvpn.test.e2e"
    TEST_APK_PATH="$APK_BASE_DIR/test/e2e/build/outputs/apk/$BILLING_FLAVOR${INFRA_FLAVOR^}/debug/e2e-$BILLING_FLAVOR-$INFRA_FLAVOR-debug.apk"
    ;;
esac

LOCAL_TMP_REPORT_PATH="/tmp/mullvad-$TEST_TYPE-instrumentation-report"
INSTRUMENTATION_LOG_FILE_PATH="$LOCAL_TMP_REPORT_PATH/instrumentation-log.txt"
LOGCAT_FILE_PATH="$LOCAL_TMP_REPORT_PATH/logcat.txt"
LOCAL_SCREENSHOT_PATH="$LOCAL_TMP_REPORT_PATH/screenshots"
DEVICE_SCREENSHOT_PATH="/sdcard/Pictures/mullvad-$TEST_TYPE"

echo ""
echo "### Ensure clean report structure ###"
rm -rf "$LOCAL_TMP_REPORT_PATH" || echo "No report path"
adb logcat --clear
adb shell rm -rf "$DEVICE_SCREENSHOT_PATH"
mkdir "$LOCAL_TMP_REPORT_PATH"
echo ""

if [[ "${USE_ORCHESTRATOR-}" == "true" ]]; then
    if [[ "${AUTO_FETCH_TEST_HELPER_APKS-}" == "true" ]]; then
        echo "### Fetching orchestrator and test services apks ###"
        ORCHESTRATOR_APK_PATH=$DEFAULT_ORCHESTRATOR_APK_PATH
        TEST_SERVICES_APK_PATH=$DEFAULT_TEST_SERVICES_APK_PATH
        curl -sL "$ORCHESTRATOR_URL" -o "$ORCHESTRATOR_APK_PATH"
        curl -sL "$TEST_SERVICES_URL" -o "$TEST_SERVICES_APK_PATH"
        echo ""
    else
        if [[ -z ${ORCHESTRATOR_APK_PATH-} ]]; then
            echo "The variable ORCHESTRATOR_APK_PATH is not set."
            exit 1
        fi
        if [[ -z ${TEST_SERVICES_APK_PATH-} ]]; then
            echo "The variable TEST_SERVICES_APK_PATH is not set."
            exit 1
        fi
    fi
fi

echo "### Ensure that packages are not previously installed ###"
adb uninstall "$PACKAGE_NAME" || echo "App package not installed"
adb uninstall "$TEST_PACKAGE_NAME" || echo "Test package not installed"
adb uninstall androidx.test.services || echo "Test services package not installed"
adb uninstall androidx.test.orchestrator || echo "Test orchestrator package not installed"
echo ""

echo "Starting instrumented tests of type: $TEST_TYPE"
echo ""

echo "### Install packages ###"
adb install -t "$APK_PATH"
adb install "$TEST_APK_PATH"
if [[ "$USE_ORCHESTRATOR" == "true" ]]; then
    echo "Using ORCHESTRATOR_APK_PATH: $ORCHESTRATOR_APK_PATH"
    adb install "$ORCHESTRATOR_APK_PATH"
    echo "Using TEST_SERVICES_APK_PATH: $TEST_SERVICES_APK_PATH"
    adb install "$TEST_SERVICES_APK_PATH"
fi
echo ""

echo "### Run instrumented test command ###"
if [[ "$USE_ORCHESTRATOR" == "true" ]]; then
    INSTRUMENTATION_COMMAND="\
    CLASSPATH=\$(pm path androidx.test.services) app_process / androidx.test.services.shellexecutor.ShellMain \
    am instrument -r -w \
    -e targetInstrumentation $TEST_PACKAGE_NAME/androidx.test.runner.AndroidJUnitRunner \
    -e clearPackageData true \
    -e runnerBuilder de.mannodermaus.junit5.AndroidJUnit5Builder \
    ${OPTIONAL_TEST_ARGUMENTS:-""} \
    androidx.test.orchestrator/androidx.test.orchestrator.AndroidTestOrchestrator"
else
    INSTRUMENTATION_COMMAND="\
    am instrument -w \
    -e runnerBuilder de.mannodermaus.junit5.AndroidJUnit5Builder \
    $TEST_PACKAGE_NAME/androidx.test.runner.AndroidJUnitRunner"
fi
adb shell "$INSTRUMENTATION_COMMAND" | tee "$INSTRUMENTATION_LOG_FILE_PATH"
echo ""

echo "### Ensure that packages are uninstalled ###"
adb uninstall "$PACKAGE_NAME" || echo "App package not installed"
adb uninstall "$TEST_PACKAGE_NAME" || echo "Test package not installed"
adb uninstall androidx.test.services || echo "Test services package not installed"
adb uninstall androidx.test.orchestrator || echo "Test orchestrator package not installed"
echo ""

echo "### Checking logs for failures ###"
if grep -q "$LOG_FAILURE_MESSAGE" "$INSTRUMENTATION_LOG_FILE_PATH"; then
    echo "One or more tests failed, see logs for more details."
    echo "Collecting report..."
    adb pull "$DEVICE_SCREENSHOT_PATH" "$LOCAL_SCREENSHOT_PATH" || echo "No screenshots"
    adb logcat -d > "$LOGCAT_FILE_PATH"
    exit 1
else
    echo "No failures!"
fi
