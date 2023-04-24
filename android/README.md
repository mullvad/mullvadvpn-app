# Mullvad VPN Android app

This directory contains the Android project as well as related files and information. Apart from the
content of this directory, the Android project also relies on building [wireguard-go](../wireguard/)
and the [mullvad-daemon](../mullvad-daemon/) which are both bundled as shared libraries into the
assembled APK.

The app is available for download on
[our website](https://mullvad.net/download/android/),
[GitHub Releases](https://github.com/mullvad/mullvadvpn-app/releases),
[F-Droid](https://f-droid.org/packages/net.mullvad.mullvadvpn/) and
[Google Play](https://play.google.com/store/apps/details?id=net.mullvad.mullvadvpn).

## Quick start

### Browsing the source code

The content in this directory (`<repository-root>/android`) follows a standard Android project
structure and can therefore be opened in Android Studio or any other IDE or editor of your choice.

### Building the app

The easiest and recommended way to build the Android project including the `mullvad-daemon` and
`wireguard-go` is to use the following command (which requires `podman`):
```
../building/containerized-build.sh android --dev-build
```
See the [build instructions](BuildInstructions.md) for further information.

## Linting and formatting

### Kotlin formatting
`ktfmt` is used for kotlin formatting.

See the [official documentation](https://github.com/facebook/ktfmt) for how to use it as default
formatter in Android Studio. Ensure to set `kotlinLangStyle` as "Code Style" and to set the project
to rely on the `EditorConfig` (`.editorconfig` file).

Also, see the [`ktfmt` gradle plugin documentation](https://github.com/cortinico/ktfmt-gradle) for
how to use it as a gradle task.

### XML formatting
In order to format XML files, the script `scripts/tidy.sh` is used. As the script name implies, it's basically a helper script to run the tool called `tidy`. It needs to be installed unless the
container image is used.

Command to format:
```
scripts/tidy.sh format
```

Command to format and check for any changes:
```
scripts/tidy.sh formatAndCheckDiff
```

#### macOS
Since macOS is using a different version of `sed` running the tidy script (`scripts/tidy.sh`) will 
lead to the creation of a large number of files ending with `-e`. The recommended fix for this 
issue is to install the gnu version of `sed`. This can be done by running:
`brew install gnu-sed` and then set `gnu-sed` as your default `sed`.

### Android Gradle Plugin lint tool

The Android Gradle Plugin lint tool for linting of resources and code. The tool is configured to be
strict. See each `build.gradle.kts`for more information.

## Translations and localization

See the [locale README][gui-locales-readme] for how to easily update translations. It also includes
documentation for which phrases and terms shouldn't be translated (Do Not Translate). Also see the
[translations converter README](translations-converter-readme) for documentation about
the tool used to sync translations between the Android and Desktop apps.

[gui-locales-readme]: ../gui/locales/README.md
[translations-converter-readme]: ./translations-converter/README.md

## Icons and assets

For a general overview of icons and graphics read [the graphics readme](../graphics/README.md).
