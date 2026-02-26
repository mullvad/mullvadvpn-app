#!/usr/bin/env bash
# Generates the screenshots needed for the website guide by calling a maestrotest script.
# Call this script with a valid production account number (does not matter which).
# This script must be run against a rooted device or emulator image.

set -euo pipefail

if [[ $# -eq 0 || "$1" == "--help" ]]; then
    echo "Usage: $0 PROD_ACCOUNT_NUMBER"
    exit
fi

ACCOUNT_NUMBER="$1"
# Insert a space after every 4 chars in the account number.
# This is required to create a new device with the API.
# shellcheck disable=SC2001
ACCOUNT_NUMBER_SPACES=$(echo "$ACCOUNT_NUMBER" | sed 's/.\{4\}/& /g')

# Make sure we have 5/5 devices so we get the Too many devices screen.
COOKIES=$(curl --request POST \
  --url https://mullvad.net/en/account/login \
  --header 'content-type: application/x-www-form-urlencoded' \
  --header 'origin: https://mullvad.net' \
  --data "account_number=$ACCOUNT_NUMBER_SPACES" \
  -s -c - -o /dev/null)

while curl --request POST \
  --url 'https://mullvad.net/en/account/devices?%2Fupload-key=' \
  --header 'content-type: application/x-www-form-urlencoded' \
  --header 'origin: https://mullvad.net' \
  --data "key=$(openssl rand -base64 32 | tr '+' 'A')" \
  -b - <<< "$COOKIES" -s | grep -q '"status":200'; do
    echo "Created new device"
    sleep 2
done

CURRENT_DIR=$(pwd)
SCRIPT_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
ANDROID_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

adb root
adb wait-for-device

# Set the device in demo mode to control the status bar
adb shell settings put global sysui_demo_allowed 1
adb shell am broadcast -a com.android.systemui.demo --es command network --es mobile hide --es wifi show --es level 4 --es fully true
adb shell am broadcast -a com.android.systemui.demo --es command clock --es hhmm 2009

cd "$ANDROID_DIR"

# Redirect output to void and return true so 'set -e' doesn't kill the script
adb uninstall net.mullvad.mullvadvpn > /dev/null 2>&1 || true
git apply "$SCRIPT_DIR/account_name_and_expiry.patch"
./gradlew installOssProdDebug
git restore lib

# Start capturing screenshot
cd "$CURRENT_DIR"
maestro test -e ACCOUNT_NUMBER="$ACCOUNT_NUMBER" "$SCRIPT_DIR/screenshot-flow.yaml"

echo "done!"

