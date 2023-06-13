#!/usr/bin/env/ bash
# Buildscript to run on our buildserver to create and upload an app.

set -eu -o pipefail

# Add SSH key for iOS build VMs to be able to SSH into them without user interaction
ssh-add ~/.ssh/ios-vm-key

VM_BUILD_DIR="/Volumes/My Shared Files/build/ios"
BUILD_SCRIPT="
security unlock-keychain -p 'build' && \
    (cd ${VM_BUILD_DIR}; bash build.sh)
"

bash run-in-vm.sh "ios-build" "${BUILD_SCRIPT}" "build:mullvadvpn-app"
mkdir -p build-output
cp mullvadvpn-app/ios/Build/MullvadVPN.ipa build-output/MullvadVPN.ipa

VM_UPLOAD_DIR="/Volumes/My Shared Files/build-output"
VM_UPLOAD_IPA_PATH="${VM_UPLOAD_DIR}/MullvadVPN.ipa"
# TODO figure out path to API key
API_KEY_PATH="~/ci/app-store-connect-key.json"
UPLOAD_SCRIPT="(cd ci/; bundle exec fastlane pilot upload --api-key ${API_KEY_PATH} --ipa ${VM_UPLOAD_IPA_PATH})"

bash run-in-vm.sh "ios-upload" "${UPLOAD_SCRIPT}" "build-output:build-output"
