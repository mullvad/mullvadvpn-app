#!/usr/bin/env bash
# Buildscript to run on our buildserver to create and upload an iOS app.
#
# The script expects two tart VMs, `ios-build` and `ios-upload` to exist and be
# executable. The build VM is expected to have the appropriate environment to
# be able to build the app and the upload VM - to upload the app. Both must be
# reachable via SSH without a password via the SSH key located in
# ~/.ssh/ios-vm-key.

set -eu -o pipefail

# Add SSH key for iOS build VMs to be able to SSH into them without user interaction
eval "$(ssh-agent)"
ssh-add ~/.ssh/ios-vm-key

# This single path really screws with XCode and wireguard-go's makefiles, which
# really do not like the whitespace. Thus, the build source is copied to a
# non-whitespaced `~/build`, built there and the resulting `MullvadVPN.ipa` is
# copied back.
MULLVAD_CHECKOUT_DIR="${MULLVAD_CHECKOUT_DIR:-mullvadvpn-app}"

bash run-in-vm.sh "ios-build" "$(pwd)/build-app.sh" "build:${MULLVAD_CHECKOUT_DIR}"
mkdir -p build-output
cp "${MULLVAD_CHECKOUT_DIR}/ios/Build/MullvadVPN.ipa" build-output/MullvadVPN.ipa

bash run-in-vm.sh "ios-upload" "$(pwd)/upload-app.sh" "build-output:build-output"
