# Android release scripts

Currently there are two release scripts that deal with different parts of the android release process.

## Github-release
Download and verifies a release from releases.mullvad.net and push it to gihub with a changelog.

### Prequisites
This script requires `sequoia-pgp` and `GitHub CLI`.

#### Install dependencies

##### MacOS
brew install sq sqv gh

##### Linux
sudo apt install sq

#### Usage
Checkout the release branch and make sure you are on the latest commit.
Then run the following command:

```bash
android/scripts/release/github-release
```

It will ask you to make any final changes to the changelog. After that it will publish a draft to github.

## Release
Add and remove supported versions and set the latest version.

### Prequisites
This script requires `rust` and access to the build server.
It is possible change supported versions and latest version without using rust by calling
`publish-metadata-to-api` directly, but this requires manually downloading and editing the metadata files.

#### Usage

##### Add supported version
To add a supported version to the list use the following command:
```bash
/android/scripts/release/release app-build-linux3 cdn.mullvad.net --add-release [version]
```
The script will display a diff, check that it looks good and press enter to continue.

##### Remove supported version
To remove a supported version from the list use the following command:
```bash
/android/scripts/release/release app-build-linux3 cdn.mullvad.net --remove-release [version]
```
The script will display a diff, check that it looks good and press enter to continue.

##### Set latest stable
To set the latest stable version use the following command:
```bash
/android/scripts/release/release app-build-linux3 cdn.mullvad.net --set-latest-stable [version]
```

**NOTE:** A version needs to be supported to be set as latest.

The script will display a diff, check that it looks good and press enter to continue.
