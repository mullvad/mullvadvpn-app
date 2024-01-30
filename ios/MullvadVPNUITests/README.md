# Integration tests

## iOS device setup
1. Make sure device is added to provisioning profiles
2. Disable passcode in iOS settings - otherwise tests cannot be started without manually entering passcode
3. Make sure device is configured in GitHub secrets(see *GitHub setup* below)

## New runner setup
1. Install Xcode
2. Sign in with Apple id in Xcode
3. Download manual provisioning profiles in Xcode
4. Install Xcode command line tools `xcode-select --install`
5. Install yeetd
 - `wget https://github.com/biscuitehh/yeetd/releases/download/1.0/yeetd-normal.pkg`
 - `sudo installer -pkg yeetd-normal.pkg -target yeetd`
6. Install Homebrew and dependencies
  - `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`
  - `brew install xcbeautify wget swiftlint`
7. Install Ruby
  - `\curl -sSL https://get.rvm.io | bash`
8. Install Rust and add iOS targets
  - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - `rustup target add install aarch64-apple-ios aarch64-apple-ios-sim`
9. Install Go 1.19 [https://go.dev/dl/](https://go.dev/dl/)

## GitHub setup
1. Ask GitHub admin for new runner token and set it up according to the steps presented, pass `--labels ios-test` to `config.sh` when running it.
2. Make sure GitHub actions secrets are correctly set up:
  - `IOS_DEVICE_PIN_CODE`
  - `IOS_HAS_TIME_ACCOUNT_NUMBER`
  - `IOS_NO_TIME_ACCOUNT_NUMBER`
  - `IOS_TEST_DEVICE_IDENTIFIER_UUID` - unique identifier for the test device. Create new identifier with `uuidgen`.
  - `IOS_TEST_DEVICE_UDID` - the iOS device's UDID.
