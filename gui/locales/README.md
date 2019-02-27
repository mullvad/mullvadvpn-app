This is a folder with gettext translations for Mullvad VPN app.

## Dependency installation notes

Make sure to install the GNU Gettext utilities.

### Linux

Normally shipped with the OS.

### macOS

Install `gettext` via Homebrew:

```
brew install gettext
```

### Windows

Please follow the downlaod instructions at https://www.gnu.org/software/gettext/


## Adding new translations

Create a new sub-folder under `gui/packages/desktop/locales`, use the locale identifier for the
folder name.

The complete list of supported locale identifiers can be found at:

https://electronjs.org/docs/api/locales

In order to initialize the translations catalogue for the new locale, simple follow the update
procedure, described in the section below.


## Updating translations

Run `yarn workspace desktop update-translations` to extract the new translations from the source
code and update all of the existing catalogues.

The new translations are automatically added to empty sub-folders using the POT template at
`gui/packages/desktop/locales/messages.pot`. Folders that contain a `.gitkeep` file are ignored.
