# Making a release

When making a real release there are a couple of steps to follow. `<VERSION>` here will denote
the version of the app you are going to release. For example `2018.3-beta1` or `2018.4`.

1. Follow the [Install toolchains and dependencies](BuildInstructions.md#install-toolchains-and-dependencies) steps
   if you have not already completed them.

1. Make sure the `CHANGELOG.md` is up to date and has all the changes present in this release.
   Also change the `[Unreleased]` header into `[<VERSION>] - <DATE>` and add a new `[Unreleased]`
   header at the top. Push this, get it reviewed and merged.

1. Run `./prepare-release.sh [--desktop] [--android] <VERSION>`. This will do the following for you:
    1. Check if your repository is in a sane state and the given version has the correct format
    1. Update `package.json` with the new version and commit that
    1. Add a signed tag to the current commit with the release version in it

    Please verify that the script did the right thing before you push the commit and tag it created.

1. When building for Windows or macOS, the following environment variables must be set:
   * `CSC_LINK` - The path to the certificate used for code signing.
      * Windows: A `.pfx` certificate.
      * macOS: A `.p12` certificate file with the Apple application signing keys.
        This file must contain both the "Developer ID Application" and the "Developer ID Installer"
        certificates + private keys.
   * `CSC_KEY_PASSWORD` - The password to the file given in `CSC_LINK`. If this is not set then
      `build.sh` will prompt you for it. If you set it yourself, make sure to define it in such a
      way that it's not stored in your bash history:
      ```bash
      export HISTCONTROL=ignorespace
      export CSC_KEY_PASSWORD='my secret'
      ```

   * *macOS only*:
      * `NOTARIZE_APPLE_ID` - The AppleId to use when notarizing the app. Only needed on release builds
      * `NOTARIZE_APPLE_ID_PASSWORD` - The AppleId password for the account in `NOTARIZE_APPLE_ID`.
         Don't use the real AppleId password! Instead create an app specific password and add that to
         your keyring. See this documentation: https://github.com/electron/electron-notarize#safety-when-using-appleidpassword

         Summary:
         1. Generate app specific password on Apple's AppleId management portal.
         2. Run `security add-generic-password -a "<apple_id>" -w <app_specific_password> -s "something_something"`
         3. Set `NOTARIZE_APPLE_ID_PASSWORD="@keychain:something_something"`.

1. Run `./build.sh` on each computer/platform where you want to create a release artifact. This will
    do the following for you:
    1. Update `relays.json` with the latest relays
    1. Compile and package the app into a distributable artifact for your platform.

    Please pay attention to the output at the end of the script and make sure the version it says
    it built matches what you want to release.
