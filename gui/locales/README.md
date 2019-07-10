This is a folder with gettext translations for Mullvad VPN app.

## Adding new translations

Create a new sub-folder under `gui/locales`, use the locale identifier for the
folder name.

The complete list of supported locale identifiers can be found at:

https://electronjs.org/docs/api/locales

In order to initialize the translations catalogue for the new locale, simple follow the update
procedure, described in the section below.


## Updating translations template

Run `npm run update-translations` to extract the new translations from the source
code and update all of the existing catalogues.

The new translations are automatically added to empty sub-folders using the POT template at
`gui/locales/messages.pot`. Folders that contain a `.gitkeep` file are ignored.

## Uploading translations template to Crowdin

After updating the translations template (POT) locally, make sure to upload it to Crowdin:

```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY ./gui/scripts/crowdin.sh upload
```

Triggering Crowdin to start translating has to be done manually. Speak to the project owner

## Downloading translations from Crowdin

Before downloading from Crowdin the project must be "built" through their web interface. When you
later download you will receive the translations from the last point in time when it was built:

Go to the Crowdin project > Settings > "Build & Download" dropdown button > "Build project"

In order to download and integrate the new translations from Crowdin into the app, use the following
command:

```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY ./gui/scripts/crowdin.sh download
```
