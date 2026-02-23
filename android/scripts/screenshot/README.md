## Overview
The `gen-guide-screenshots.sh` script in this folder can be used to generate the app screenshots
that are needed for the user guide at https://mullvad.net/en/help/using-mullvad-vpn-on-android

## Setup
Before running the script, do the following:
1. Make sure you have a rooted device or emulator. To create a rooted emulator, select `Google APIs`
   instead of the default `Google Play Store` option in the `Services` dropdown menu when
   creating the device.
2. Make sure that you have `adb` on your `PATH`.
3. Install the maestro cli and make sure that the `maestro` binary is on your `PATH`.
   See https://docs.maestro.dev/maestro-cli/how-to-install-maestro-cli

## Running
In a terminal, change to the directory where you want the screenshots to be put. Then run the
following:
`gen-guide-screenshots.sh [YOUR PROD ACCOUNT NUMBER]`
