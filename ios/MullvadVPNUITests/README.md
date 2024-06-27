# iOS end to end tests
## Running tests
### Locally using Xcode
Tests can be triggered locally from Xcode in the Test navigator or by running tests from the diamond in the editor gutter.

#### On GitHub
There are five workflows running tests:
 - [ios-end-to-end-tests.yml](https://github.com/mullvad/mullvadvpn-app/actions/workflows/ios-end-to-end-tests.yml) - super workflow which other workflows reuse. This is also the workflow you can manually trigger to run all tests or optionally specify which tests to run.
 - [ios-end-to-end-tests-nightly.yml](https://github.com/mullvad/mullvadvpn-app/actions/workflows/ios-end-to-end-tests-nightly.yml) - scheduled nightly test run, running all tests.
 - [ios-end-to-end-tests-merge-to-main.yml](https://github.com/mullvad/mullvadvpn-app/actions/workflows/ios-end-to-end-tests-merge-to-main.yml) - automatically tryggered by a PR merge to `main`.
 - [ios-end-to-end-tests-api.yml](https://github.com/mullvad/mullvadvpn-app/actions/workflows/ios-end-to-end-tests-api.yml) - manually triggered tests focusing on making sure the API is functioning as intended on stagemole.
 - [ios-end-to-end-tests-settings-migration.yml](https://github.com/mullvad/mullvadvpn-app/actions/workflows/ios-end-to-end-tests-settings-migration.yml) - for now this is still manually triggered. Tests installing older version of the app, changing settings, upgrading the app and verifying that settings were correctly migrated.

## Adding more tests
When adding more files with test suites they must be added to the `MullvadVPNUITestsAll` test plan and also added to the appropriate node(s) in `ios/MullvadVPNUITests/tests.json` file in order to run in CI. For new test cases in already existing test suite nothing needs to be done. The test case/suite values in `tests.json` translate to input for `xcodebuild -only-testing` which is in the format `<target-name>/<test-suite-name>/<test-case-name>`. The GitHub actions workflow will add the `<target-name>` part so only `<test-suite-name>/<test-case-name>` is required, where `<test-case-name>` is optional. So for example `AccountTests` and `AccountTests/testLogin` are both valid values.

## Set up local environment
To run tests locally you need to make sure you have copied the configuration template `UITests.xcconfig.template` to `UITests.xcconfig` and set up the configuration attributes. The configuration attributes you're mostly likely to want to set custom values for are at the top:
```
// Pin code of the iOS device under test
IOS_DEVICE_PIN_CODE = 

// UUID to identify test runs. Should be unique per test device. Generate with for example uuidgen on macOS.
TEST_DEVICE_IDENTIFIER_UUID = 
```

Look through other configuration attributes as well, but it is likely that their default value should be kept. Default values are set with local test execution in mind. They are changed in CI.

## CI setup
### iOS device setup
1. Make sure device is added to provisioning profiles
2. Enable developer mode
3. Disable passcode in iOS settings - otherwise tests cannot be started without manually entering passcode
4. Set the value of `TEST_DEVICE_UDID` to the UDID of the test device in `ios-end-to-end-tests.yml` and `ios-end-to-end-tests-settings-migration.yml`.
5. Make sure the test device is connected to the WiFi `app-team-ios-tests`
6. Make sure iCloud syncing of keychain is off on the device so that the device isn't getting WiFi passwords from another device causing it to sometimes connect to another WiFi.
7. After the device is set up download updated provisioning profiles on the GitHub runner computer(Download manual profiles in Xcode settings)

### Set up of runner build environment
1. Install Xcode
2. Sign in with Apple id in Xcode
3. Download manual provisioning profiles in Xcode
4. Install Xcode command line tools `xcode-select --install`
5. Install yeetd
 - `wget https://github.com/biscuitehh/yeetd/releases/download/1.0/yeetd-normal.pkg`
 - `sudo installer -pkg yeetd-normal.pkg -target yeetd`
6. Install ios-deploy and jq
  - `brew install ios-deploy jq`
7. Install Homebrew and dependencies
  - `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`
  - `brew install xcbeautify wget swiftlint protobuf`
8. Install Ruby
  - `\curl -sSL https://get.rvm.io | bash`
9. Install Rust and add iOS targets
  - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - `rustup target install aarch64-apple-ios aarch64-apple-ios-sim`
10. Install Go 1.19
  - `brew install go@1.19`

### GitHub runner setup
1. Ask GitHub admin for new runner token and setup steps from GitHub. Set it up according to the steps, pass `--labels ios-test` to `config.sh` when running it. By default it will also have the labels `self-hosted` and `macOS` which are required as well.
2. Make sure GitHub actions secrets for the GitHub project are correctly set up:
  - `IOS_DEVICE_PIN_CODE` - Device passcode if the device require it, otherwise leave blank. Devices used with CI should not require passcode.
  - `IOS_HAS_TIME_ACCOUNT_NUMBER` - Production server account without time left
  - `IOS_NO_TIME_ACCOUNT_NUMBER` - Production server account with time added to it
  - `TEST_DEVICE_IDENTIFIER_UUID` - unique identifier for the test device. Create new identifier with `uuidgen`.
  - `PARTNER_API_TOKEN` - secret token for partner API. Optional and only intended to be used in CI when running tests against staging environment.

### Specifying which tests run when in CI
Which tests run when is specified in `tests.json`(See _Adding more tests_). Settings migration is an exception, it uses four different test plans and a separate workflow `ios-end-to-end-tests-settings-migration.yml` which executes the test plans in order, do not reinstall the app in between runs but upgrades the app after changing settings:
* `MullvadVPNUITestsChangeDNSSettings` - Change settings for using custom DNS
* `MullvadVPNUITestsVerifyDNSSettingsChanged` - Verify custom DNS settings still changed
* `MullvadVPNUITestsChangeSettings` - Change all settings except custom DNS setting
* `MullvadVPNUITestsVerifySettingsChanged` - Verify all settings except custom DNS setting still changed
