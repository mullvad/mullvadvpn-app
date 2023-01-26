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

## Translations and localization

See the [locale README][gui-locales-readme] for how to easily update translations. It also includes
documentation for which phrases and terms shouldn't be translated (Do Not Translate). Also see the
[translations converter README](translations-converter-readme) for documentation about
the tool used to sync translations between the Android and Desktop apps.

[gui-locales-readme]: ../gui/locales/README.md
[translations-converter-readme]: ./translations-converter/README.md

## Icons and assets

For a general overview of icons and graphics read [the graphics readme](../graphics/README.md).
