# Installer downloader

This directory contains the source code for a minimal downloader/installer for the Mullvad VPN app.

## Making a release

This section describes how to build and release the downloader itself. Note that this is totally
unrelated to releasing new versions of the Mullvad VPN app.

The installer is built locally and then uploaded to `app-build-linux`, which runs
`buildserver-upload.sh`. `buildserver-upload.sh` will take care of GPG signing it and publishing to
CDNs.

`<version>` below will be used below to refer to the version that is being released.

### Prerequisites

1. Install Rust if you do not have it. It can be obtained from https://rustup.rs/

1. You'll typically need a Windows and a macOS machine, since the installer must be built on both.

1. Set up macOS and Windows for signing:

    * macOS: `CSC_LINK` must point to the path of a `.p12` certificate used for code signing.

    * Windows: `CERT_HASH` must be set to the fingerprint of the signing certificate to use. This
               must be present in the certificate store.

    See the [app release instructions](../Release.md) for more details on this.

1. Set up `~/.ssh/config` for `app-build-linux`. Build artifacts will be `sftp`'d here.

### Updating the version

1. Bump the version in `Cargo.toml`.

1. Create a signed tag called `desktop/installer-downloader/<version>` and `git push` the tag.

### Build and upload

Perform this step on both macOS and Windows:

1. Run `./build.sh --upload --sign`. This will build a release build of the installer and upload it
   to `app-build-linux:upload/installer-downloader/<version>`
