# Translations converter tool

A tool for helping two-way sync the translations between the Android and Desktop apps.

## How to run

Run the following command (requires `rust` and `cargo` which can be installed using [rustup.rs](https://rustup.rs/)):
```bash
cargo run
```

## Translations files

The tool creates the appropriate localized message files and directories under the
[Android project resources](android-resources) (e.g. [values-sv/strings.xml](values-sv-example))
for each locale it can find in the [`gui/locales` directory][gui-locales]. It will also update the
[messages.pot] template file with the string messages from the Android app for which it did not find
any translation, making it simpler to use the template for obtaining those translations.

[android-resources]: ../app/src/main/res/
[gui-locales]: ../gui/locales/
[messages.pot]: ../gui/locales/messages.pot
[values-sv-example]: ../app/src/main/res/values-sv/strings.xml
