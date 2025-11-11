# iOS end to end tests
## Running tests
### Locally using Xcode
Tests can be triggered locally from Xcode in the Test navigator or by running tests from the diamond in the editor gutter.

#### On GitHub
There are five workflows running tests:
 - [ios-end-to-end-tests.yml](https://github.com/mullvad/mullvadvpn-app/actions/workflows/ios-end-to-end-tests.yml) - super workflow which other workflows reuse. This is also the workflow you can manually trigger to run all tests or optionally specify which tests to run.
 - [ios-end-to-end-tests-nightly.yml](https://github.com/mullvad/mullvadvpn-app/actions/workflows/ios-end-to-end-tests-nightly.yml) - scheduled nightly test run, running all tests.
 - [ios-end-to-end-tests-merge-to-main.yml](https://github.com/mullvad/mullvadvpn-app/actions/workflows/ios-end-to-end-tests-merge-to-main.yml) - automatically triggered by a PR merge to `main`.
 - [ios-end-to-end-tests-api.yml](https://github.com/mullvad/mullvadvpn-app/actions/workflows/ios-end-to-end-tests-api.yml) - manually triggered tests focusing on making sure the API is functioning as intended on stagemole.

## Adding more tests
When adding more files with test suites they must be added to the `MullvadVPNUITestsAll` test plan and also added to the appropriate node(s) in `ios/MullvadVPNUITests/tests.json` file in order to run in CI. For new test cases in already existing test suite nothing needs to be done. The test case/suite values in `tests.json` translate to input for `xcodebuild -only-testing` which is in the format `<target-name>/<test-suite-name>/<test-case-name>`. The GitHub actions workflow will add the `<target-name>` part so only `<test-suite-name>/<test-case-name>` is required, where `<test-case-name>` is optional. So for example `AccountTests` and `AccountTests/testLogin` are both valid values.

## Set up local environment
To run tests locally you need to make sure you have copied the configuration template `UITests.xcconfig.template` to `UITests.xcconfig` and set up the configuration attributes. The configuration attributes you're mostly likely to want to set custom values for are at the top:
```
// Pin code of the iOS device under test.
IOS_DEVICE_PIN_CODE =

// UUID to identify test runs. Should be unique per test device. Generate with for example uuidgen on macOS.
TEST_DEVICE_IDENTIFIER_UUID =
```

Look through other configuration attributes as well, but it is likely that their default value should be kept. Default values are set with local test execution in mind. They are changed in CI.

The test device must be on the office WiFi `app-team-ios-tests` in order to be able to run tests making use of the firewall and packet capture APIs.

## CI setup
### iOS device setup
1. Make sure device is added to provisioning profiles
2. Enable developer mode
3. Disable passcode in iOS settings - otherwise tests cannot be started without manually entering passcode
4. Set the value of `TEST_DEVICE_UDID` to the UDID of the test device in `ios-end-to-end-tests.yml`.
5. Make sure the test device is connected to the WiFi `app-team-ios-tests`
6. Make sure iCloud syncing of keychain is off on the device so that the device isn't getting WiFi passwords from another device causing it to sometimes connect to another WiFi.
7. After the device is set up download updated provisioning profiles on the GitHub runner computer(Download manual profiles in Xcode settings)

### Set up of runner build environment
1. Install Xcode
2. Sign in with Apple ID in Xcode
3. Download manual provisioning profiles in Xcode
4. Install Xcode command line tools `xcode-select --install`
5. Install yeetd
 - `wget https://github.com/biscuitehh/yeetd/releases/download/1.0/yeetd-normal.pkg`
 - `sudo installer -pkg yeetd-normal.pkg -target yeetd`
6. Install ios-deploy and jq
  - `brew install ios-deploy jq`
7. Install Homebrew and dependencies
  - `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`
  - `brew install xcbeautify wget protobuf`
8. Install Ruby
  - `curl -sSL https://get.rvm.io | bash`
9. Install Rust and add iOS targets
  - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - `rustup target install aarch64-apple-ios aarch64-apple-ios-sim`
10. Install Go 1.21
  - `brew install go@1.21`

### GitHub runner setup
1. Ask GitHub admin for new runner token and setup steps from GitHub. Set it up according to the steps, pass `--labels ios-test` to `config.sh` when running it. By default it will also have the labels `self-hosted` and `macOS` which are required as well.
2. Make sure GitHub actions secrets for the GitHub project are correctly set up:
  - `IOS_DEVICE_PIN_CODE` - Device passcode for the device you want to run tests on, otherwise leave blank. Devices used with CI should not require passcode.
  - `IOS_HAS_TIME_ACCOUNT_NUMBER` - Production server account with time added to it.
  - `IOS_NO_TIME_ACCOUNT_NUMBER` - Production server account with no time. Make sure that the account has not been deleted if left unused for too long.
  - `TEST_DEVICE_IDENTIFIER_UUID` - Unique identifier for the test device. Create new identifier with `uuidgen`.
  - `PARTNER_API_TOKEN` - Secret token for partner API. Optional and only intended to be used in CI when running tests against staging environment.
  - `IOS_IN_APP_PURCHASE_USERNAME` - Secret account username for making in-app purchases. Only intended to be used in CI when running tests against staging environment.
  - `IOS_IN_APP_PURCHASE_PASSWORD` - Secret account password for making in-app purchases. Only intended to be used in CI when running tests against staging environment.

### Specifying which tests run when in CI
Which tests run when is specified in `tests.json`(See _Adding more tests_).

### Current test devices
Currently we are using an iPhone 15 Pro(UDID `00008130-0019181022F3803A`) running iOS 17.3.1.

## APIs used
The iOS team NUC is hosting APIs consumed by tests:

 - **Firewall API** - for creating temporary firewall rules blocking certain traffic. Hosted on NUC.
 - **Packet capture API** - for recording packet captures. Outputs both PCAP file and PCAP parsed to JSON(with some limitations). Hosted on NUC.
 - **Partner API** - The partner API is used on stagemole for adding time to accounts. Hosted by infra.
 - **App API** - The app API is used for managing accounts - creating, deleting, getting account info etc. Hosted by infra.

## Network setup
The NUC is hosting a WiFi which test devices need to be on in order to be able to access the firewall and packet capture APIs. The SSID is `app-team-ios-tests`. The APIs running on the NUC are accessed by using IP address `8.8.8.8` and port `80` from test devices. This is a workaround for local network access not working from UI tests. `8.8.8.8` which is a public IP address is re-routed to the NUC. This way we don't need to allow local network access in order to access the local NUC.
## Troubleshooting
### Restarting services
The easiest way to restart test services running on the NUC is by SSH:ing into it at `192.168.105.1` as `root` (password is written on a sticker under it) and rebooting `sudo shutdown -r now`.

## Gotchas
### GitHub actions concurrency
The way concurrency with GitHub actions work is that multiple workflows run concurrently, but jobs don't run concurrently. So for example if two test workflows are triggered at the same time both will start at the same time, but only one will run the `build` job at a time. When one job has finished its `build` job it will start its `test` job and the other workflow run will start its `build` job.

To make the test workflows not clash with each other the jobs output files to `~/workflow-outputs`. They create a directory which is unique for the test run, and after the test run finished the directory is removed. This is necessary because we cannot depend on the state of the working directory, since if we did test runs would be changing the working directory for each other.

### Packet capture API timeout
Tests always attempt to stop packet capture, but there is no guarantee that it can always be stopped. For example when running tests locally and stopping test execution mid packet capture the test cannot stop the packet capture. So the packet capture API has a timeout (5 minutes) in place. If a packet capture session exceeds this duration it will be stopped. This means that tests cannot do packet capture exceeding this time limit.
