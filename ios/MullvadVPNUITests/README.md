# Integration tests

## iOS device setup
1. Make sure device is added to provisioning profiles
2. Disable passcode in iOS settings - otherwise tests cannot be started without manually entering passcode
3. Make sure device is configured in GitHub secrets(see *GitHub setup* below)

## Set up of runner environment
1. Install Xcode
2. Sign in with Apple id in Xcode
3. Download manual provisioning profiles in Xcode
4. Install Xcode command line tools `xcode-select --install`
5. Install yeetd
 - `wget https://github.com/biscuitehh/yeetd/releases/download/1.0/yeetd-normal.pkg`
 - `sudo installer -pkg yeetd-normal.pkg -target yeetd`
6. Install ios-deploy
  - `brew install ios-deploy`
7. Install Homebrew and dependencies
  - `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`
  - `brew install xcbeautify wget swiftlint`
8. Install Ruby
  - `\curl -sSL https://get.rvm.io | bash`
9. Install Rust and add iOS targets
  - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - `rustup target install aarch64-apple-ios aarch64-apple-ios-sim`
10. Install Go 1.19
  - `brew install go@1.19`

## GitHub runner setup
1. Ask GitHub admin for new runner token and setup steps from GitHub. Set it up according to the steps, pass `--labels ios-test` to `config.sh` when running it. By default it will also have the labels `self-hosted` and `macOS` which are required as well.
2. Make sure GitHub actions secrets for the repository are correctly set up:
  - `IOS_DEVICE_PIN_CODE` - Device passcode if the device require it, otherwise leave blank. Devices used with CI should not require passcode.
  - `IOS_HAS_TIME_ACCOUNT_NUMBER` - Production server account without time left
  - `IOS_NO_TIME_ACCOUNT_NUMBER` - Production server account with time added to it
  - `IOS_TEST_DEVICE_IDENTIFIER_UUID` - unique identifier for the test device. Create new identifier with `uuidgen`.
  - `IOS_TEST_DEVICE_UDID` - the iOS device's UDID.

## Test plans
There are a few different test plans which are mainly to be triggered by GitHub action workflows but can also be triggered manually with Xcode:
* `MullvadVPNUITestsAll` - All tests except settings migration tests which are in separate test plan and workflow
* `MullvadVPNUITestsSmoke` - A few tests for smoke testing when merge:ing to `main`

And also the following test plans which are used for testing settings migration(`ios-end-to-end-tests-settings-migration`):

* `MullvadVPNUITestsChangeDNSSettings` - Change settings for using custom DNS
* `MullvadVPNUITestsVerifyDNSSettingsChanged` - Verify custom DNS settings still changed
* `MullvadVPNUITestsChangeSettings` - Change all settings except custom DNS setting
* `MullvadVPNUITestsVerifySettingsChanged` - Verify all settings except custom DNS setting still changed
