# Mullvad VPN Android app

This directory contains the files specific to the Android app.

## Translations / Localization

### How to update translations

See [/gui/locales/README.md][gui-locales-readme] for how to easily update translations.

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
