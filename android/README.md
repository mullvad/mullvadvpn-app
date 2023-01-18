# Mullvad VPN Android app

This directory contains the Android project as well as related files and information. Apart from the
content of this directory, the Android project also relies on [wireguard-go](../wireguard/) and the
[mullvad-daemon](../mullvad-daemon/) which are both bundled as shared libraries into the assembled
APK.

## Building the app

See the [build instructions](BuildInstructions.md) for help building the app.

## Translations / Localization

### How to update translations

See [/gui/locales/README.md][gui-locales-readme] for how to easily update translations. It also
includes documentation for which phrases and terms shouldn't be translated (Do Not Translate).

### Detailed structure and script documentation

The app has localized messages stored in `src/main/res/values-<locale>/` directories, where
`<locale>` is a two letter locale and can be followed by a two letter region code. For example: `en`
or `en-rGB`.

The translated strings are based on the gettext translation files used for the desktop app. A helper
tool is available to create the translated string resource files based on the gettext translations,
in the `translations-converter` directory. The tool can be executed with the following commands
(assuming Rust and Cargo are installed, if not, follow the steps in the [root README][root-readme]):

```
cd translations-converter
cargo run
```

After the tool finishes executing, it creates the appropriate localized message files and
directories for each locale it can find in the [`gui/locales` directory][gui-locales]. It will also
update the [messages.pot] template file with the string messages from the Android app for which it
did not find any translation, making it simpler to use the template for obtaining those
translations.

[root-readme]: ../README.md
[gui-locales-readme]: ../gui/locales/README.md
[gui-locales]: ../gui/locales/
[messages.pot]: ../gui/locales/messages.pot
