This is a folder with gettext translations for Mullvad VPN app.

## Adding new translations

1. Follow these steps to add support for a new localization language in the Xcode project:

   1. **Open the Project Settings**
      In Xcode, select the project file (`MullvadVPN.xcodeproj`) in the Project Navigator, then select the **project** (not a target).

   2. **Go to the Localizations Section**
      Under the **Info** tab, find the **Localizations** section.

   3. **Add a New Language**
      * Click the **“+”** button.
      * Select the language you want to add from the list.
      
   4. Update Localizable.xcstrings
      1. Add New Strings Through Code
         * Add a new localized string in Swift:
            ```text
            NSLocalizedString("welcome_message", comment: "Shown on the home screen")
            ```
         * or in SwiftUI:
            ```text
            Text("welcome_message")
            ```
      2. Xcode will detect that the key does not exist yet and add it to `Localizable.xcstrings` automatiacally once the project is built.

   5. **Verify Build Settings**
      Ensure the target’s **Localizable.xcstrings** list includes the new language

   6. **Run the App**
      * Open **Settings → Language** on the simulator or device.
      * Switch to the new language.
      * Verify translations appear correctly if they are translated; otherwise it shows the base language.

---

2. Add a new language on Crowdin under Settings -> Translations -> Target languages menu.

   By default the file structure is configured to produce folders with translations using two-letter
   language code (defined under Settings -> Files -> <FILE> -> ... [ellipsis] -> Settings).

   If you wish to add a dialect (i.e: `pt-BR`), you have to provide a custom mapping
   to tell Crowdin to output Portuguese (Brazil) as `pt-BR` instead of `pt`.

   In order to add a language mapping, go to Settings -> General Settings -> Language mapping
   (three faders icon on the left hand side of the "Translations" menu).


## Uploading translations template to Crowdin

After updating the strings locally, make sure to upload it to Crowdin:
```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY `./scripts/ios-localization upload`
```

Triggering Crowdin to start translating has to be done manually. Speak to the project owner

## Downloading translations from Crowdin

When the translations are done, download it by running:
```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY `./scripts/ios-localization download`
```

## Do Not Translate

All user facing phrases and terms should be translated except for the following trademarks and
names of technologies:
* Mullvad VPN
* WireGuard
* OpenVPN
* Split Tunneling
* System Transparency
* DAITA
