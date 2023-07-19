#!/usr/bin/env bash
# Upload script to run in `ios-upload` VM to upload a newly built IPA to TestFlight
set -eu -o pipefail

VM_UPLOAD_IPA_PATH="/Volumes/My\ Shared\ Files/build-output/MullvadVPN.ipa"
API_KEY_PATH="~/ci/app-store-connect-key.json"
cd ci/
source ~/.zshrc
source ~/.zprofile
bundle exec fastlane pilot upload --api-key-path ${API_KEY_PATH} --ipa ${VM_UPLOAD_IPA_PATH}
