#!/usr/bin/env/ bash
# Buildscript to run on our buildserver to create and upload an app.

set -eu -o pipefail

# Add SSH key for iOS build VMs to be able to SSH into them without user interaction
ssh-add ~/.ssh/ios-vm-key

VM_BUILD_DIR="\"/Volumes/My Shared Files/build\""

BUILD_SCRIPT="
	set -eu
	security unlock-keychain -p 'build' 
	rm -rf ~/build 
	# copying over build directory because the spaces in "/Volmes/My Shared Files/" fuck up Make and XCode
	cp -r ${VM_BUILD_DIR} ~/build || true
	cd ~/build/ios
	rm -r Build
	IOS_PROVISIONING_PROFILES_DIR=~/provisioning-profiles bash build.sh 
	cp ~/build/ios/Build/MullvadVPN.ipa /Volumes/My\ Shared\ Files/build/ios/Build/
"


bash run-in-vm.sh "ios-build" "${BUILD_SCRIPT}" "build:mullvadvpn-app"
mkdir -p build-output
cp mullvadvpn-app/ios/Build/MullvadVPN.ipa build-output/MullvadVPN.ipa

VM_UPLOAD_IPA_PATH="/Volumes/My\ Shared\ Files/build-output/MullvadVPN.ipa"
# TODO figure out path to API key
API_KEY_PATH="~/ci/app-store-connect-key.json"
UPLOAD_SCRIPT="(
	cd ci/
	source ~/.zshrc
	source ~/.zprofile
	bundle exec fastlane pilot upload --api-key-path ${API_KEY_PATH} --ipa ${VM_UPLOAD_IPA_PATH}
)"

bash run-in-vm.sh "ios-upload" "${UPLOAD_SCRIPT}" "build-output:build-output"
