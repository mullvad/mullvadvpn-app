This is a folder with gettext translations for Mullvad VPN app.

## Adding new translations

1. Create a new sub-folder under `gui/locales`, use the locale identifier for the folder name.
   
   The complete list of supported locale identifiers can be found at:
   
   https://electronjs.org/docs/api/locales

1. Add a new language on Crowdin under Settings -> Translations -> Target languages menu. 
   
   By default the file structure is configured to produce folders with translations using two-letter
   language code (defined under Settings -> Files -> <FILE> -> ... [ellipsis] -> Settings). 
  
   If you wish to add a dialect (i.e: `pt-BR`), you have to provide a custom mapping 
   to tell Crowdin to output Portuguese (Brazil) as `pt-BR` instead of `pt`.
   
   In order to add a language mapping, go to Settings -> General Settings -> Language mapping 
   (three faders icon on the left hand side of the "Translations" menu).

1. Follow the procedure as described in `gui/scripts/README.md`.

1. Optional: Upload the automatically translated `<NEW_LOCALE>/relay-locations.po` to 
   Crowdin. 
   
   *Note: Replace `<NEW_LOCALE>` with the identifier of a newly added language.*
   
   1. ZIP file with the following command:
   
      ```
      cd gui/locales
      zip payload.zip <NEW_LOCALE>/relay-locations.po
      ```
   
   1. Upload `payload.zip` to Crowdin via web interface (Settings -> Translations -> Upload 
      translations).

1. Add the language to `SUPPORTED_LOCALE_LIST` in `app.tsx`.

## Sync localizations

Use the localization script to sync localizations by running the following command from the
root-directory:
```
./scripts/localization sync-local-files
```

It will sync `messages.pot` with localization strings in the desktop app and Android app to ensure
all local files are in sync.

## Prepare strings for Crowdin translation

Use the localization script to prepare the pot-files by running the following command from the
root-directory:
```
./scripts/localization prepare
```

It will sync `messages.pot` with localization strings in the desktop app and Android app, and will
update `relay-localizations.pot`. The changes to each file will also be committed individually.

## Uploading translations template to Crowdin

After updating the translations template (POT) locally, make sure to upload it to Crowdin:
```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY `./scripts/localization upload`
```

Triggering Crowdin to start translating has to be done manually. Speak to the project owner

## Downloading translations from Crowdin

When the translations are done, download it by running:
```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY `./scripts/localization download`
```

## Verify translation formatting

Use the localization script to verify that the strings are valid HTML and that they contain the
correct amount of format specifiers:
```
./scripts/localization verify
```

## Keeping messages.pot in sync

This is only relevant when running the different tools for updating `messages.pot` manually, and
is not relevant when using the localization script mentioned above.

It's important that `messages.pot` reflect both the desktop app and the Android app. To prevent it
from getting out of sync with the strings in the source code, always run both
`npm run update-translations` and the `translations-converter` tool in that order. If the first one
is run on it's own it will remove the strings specific to Android. The easiest way to accomplish
this is to just run `./scripts/localization prepare` as described above.

## Do Not Translate

All user facing phrases and terms should be translated except for the following trademarks and
names of technologies:
* Mullvad VPN
* WireGuard
* OpenVPN
* Split Tunneling
* System Transparency
