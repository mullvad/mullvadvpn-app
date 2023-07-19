#!/usr/bin/env bash
# Buildscript to run inside a build VM to build a new IPA for the iOS app.

# This single path really screws with XCode and wireguard-go's makefiles, which
# really do not like the whitespace. Thus, the build source is copied to a
# non-whitespaced `~/build`, built there and the resulting `MullvadVPN.ipa` is
# copied back.
set -eu

VM_BUILD_DIR="\"/Volumes/My Shared Files/build\""

security unlock-keychain -p 'build'

rm -rf ~/build
cp -r ${VM_BUILD_DIR} ~/build || true
cd ~/build/ios
rm -r Build

# Instantiate Xcconfig templates.
for file in ./Configurations/*.template ; do cp $file ${file//.template/} ; done

IOS_PROVISIONING_PROFILES_DIR=~/provisioning-profiles \
    PATH=/usr/local/go/bin:$PATH \
    bash build.sh

cp ~/build/ios/Build/MullvadVPN.ipa /Volumes/My\ Shared\ Files/build/ios/Build/
