#!/usr/bin/env bash
# Upload script to run in `ios-upload` VM to upload a newly built IPA to TestFlight

VM_UPLOAD_IPA_PATH="/Volumes/My Shared Files/build-output/MullvadVPN.ipa"
API_KEY_PATH="$HOME/ci/app-store-connect-key.json"
cd ci/ || exit
# shellcheck source=/dev/null
source ~/.bash_profile
bundle exec fastlane pilot upload --api-key-path "${API_KEY_PATH}" --ipa "${VM_UPLOAD_IPA_PATH}"
