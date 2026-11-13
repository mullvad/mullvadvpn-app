# Translations

Tooling for managing the Android app translations in [Crowdin][crowdin].

Run `./localization --help` for usage.

App strings and plurals are managed directly as Crowdin sources, see
`crowdin.yml` for configuration.

Relay locations are not managed here. They are translated with shared
relay-locations.pot and downloaded via a Crowdin bundle, see the
`download-relay-locations` command in the localization script.

## Prerequisites

- [Crowdin CLI][crowdin-cli] 3.11 or later
- The `CROWDIN_API_KEY` environment variable set to a Crowdin API token

### API key

To the create the API key do the following:
- Login to [Crowdin][crowdin].
- Go to [API][crowdin-api-key] settings page
- Click Personal Access Token - New Token
- Set a name E.g "Mullvad localization script"
- Set a reasonable expiry
- Select the following permissions:
    - Projects - Source files & strings - Read & Write
    - Projects - Translations - Read only
    - Enable Granular Access and select "Mullvad App" project
- Press Create


[crowdin]: https://crowdin.com
[crowdin-api-key]: https://crowdin.com/settings#api-key
[crowdin-cli]: https://github.com/crowdin/crowdin-cli
