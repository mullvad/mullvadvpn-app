This is a folder with gettext translations for Mullvad VPN app.

## Adding new translations

1. Create a new sub-folder under `gui/locales`, use the locale identifier for the folder name.
   
   The complete list of supported locale identifiers can be found at:
   
   https://electronjs.org/docs/api/locales

1. Add a new language on Crowdin under Project Settings -> Translations menu. Make sure that the
   added language has the same identifier as the one expected by Electron.
   
   If the identifier for the same language does not match the one listed in Electron documentation,
   then remap it using the "Language mapping" on Crowdin (Located under general settings - tumblers
   icon on the left hand side from "Translations" menu).

1. Run the `extract-geo-data.py` script as described in `gui/scripts/README.md`. This will
   create the automatic translations for the new language if possible (`relay-locations.po`).

1. Upload automatically translated `gui/scripts/out/locales/$NEW_LOCALE/relay-locations.po` to 
   Crowdin via web interface: Settings -> Translations -> Upload translations, where the 
   `$NEW_LOCALE` is the identifier of the newly added language.
   
   Crowdin accepts ZIP archives with the same layout as the `gui/locales` folder.
   So make sure to ZIP the folder with the new locale, i.e the ZIP file should have the following
   file structure: `$NEW_LOCALE/relay-locations.po`.

## Updating translations template

### messages.pot

Run `npm run update-translations` to extract the new translations from the source
code. Use `crowdin.sh upload` to submit them to Crowdin.

### relay-locations.pot

To update the countries and cities you have to run the geo data scripts. Follow the instructions
in `gui/scripts/README.md`.

## Uploading translations template to Crowdin

After updating the translations template (POT) locally, make sure to upload it to Crowdin:

```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY ./gui/scripts/crowdin.sh upload
```

Triggering Crowdin to start translating has to be done manually. Speak to the project owner

## Downloading translations from Crowdin

Before downloading from Crowdin the project must be "built" first. When you
later download you will receive the translations from the last point in time when it was built.

In order to make a fresh build with translations, use the following command:

```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY ./gui/scripts/crowdin.sh export
```

In order to download and integrate the new translations from Crowdin into the app, use the following
command:

```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY ./gui/scripts/crowdin.sh download
```
