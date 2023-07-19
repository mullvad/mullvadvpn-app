# Automating iOS builds
Building an app on iOS is a hodge-podge of 2 VMs and too many bash scripts.
It all starts with `buildserver-build-ios.sh`. It does 2 main things:
- polls the app repo
- tries to build _well signed_ tags that match `^ios/` with `run-build-and-upload.sh`.

_Well signed_ in this case implies that the tag has been signed by a GPG key that is signed by
Mullvad's code signing key.


## `run-build-and-upload.sh`
This script builds the iOS app archive in one VM, which has our app signing keys, and uploads the
resulting archive in the upload VM, which has our AppStoreConnect API keys. This segregation is
intentional, compromising the build VM won't allow an attacker to take down our app store listing,
compromising the upload VM won't allow the attacker to compromise our signing keys.

The script uses `run-in-vm.sh` to execute scripts inside these 2 VMs:
- `build-app.sh` is executed in the build VM, which eventually executes `ios/build.sh`.
- `upload-app.sh` is executed in the upload VM

