## Adding New Translations

1. Follow these steps to add support for a new language in the Xcode project:

   1. **Open the Project Settings**
      In Xcode, select the project file (`MullvadVPN.xcodeproj`) in the Project Navigator, then select the **project** (not a target).

   2. **Go to the Localizations Section**
      Under the **Info** tab, locate the **Localizations** section.

   3. **Add a New Language**
      * Click the **“+”** button.
      * Select the language you want to add from the list.

   4. **Update Localizable.xcstrings**
      1. **Add New Strings Through Code**

         * Add a new localized string in Swift:

           ```
           NSLocalizedString("welcome_message", comment: "Shown on the home screen")
           ```
         * Or in SwiftUI:

           ```
           Text("welcome_message")
           ```
      2. Xcode will detect that the key does not exist yet and will add it to `Localizable.xcstrings` automatically once the project is built.

   5. **Verify Build Settings**
      Ensure the target’s **Localizable.xcstrings** list includes the new language.

   6. **Run the App**

      * Open **Settings → Language** on the simulator or device.
      * Switch to the new language.
      * Verify that translations appear correctly. If a string is not translated, the base language will be shown.

---

2. Add a new language on Crowdin under **Settings → Translations → Target languages**.

   By default, the file structure is configured to produce folders using a two-letter language code
   (defined under **Settings → Files → <FILE> → … → Settings**).

   If you want to add a dialect (e.g., `pt-BR`), you must provide a custom mapping to tell Crowdin to output Portuguese (Brazil) as `pt-BR` instead of `pt`.

   To add a language mapping, go to **Settings → General Settings → Language mapping**
   (the three-faders icon on the left side of the “Translations” menu).

## Uploading the Translation Template to Crowdin

After updating the strings locally, upload them to Crowdin:

```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY ./scripts/ios-localization upload
```

Triggering translations in Crowdin must be done manually. Contact the project owner.

## Downloading Translations from Crowdin

When translations are ready, download them by running:

```
CROWDIN_API_KEY=$YOUR_CROWDIN_KEY ./scripts/ios-localization download
```

## Do Not Translate

All user-facing phrases and terms should be translated **except** for the following trademarks and technology names:

* Mullvad VPN
* WireGuard
* OpenVPN
* Split Tunneling
* System Transparency
* DAITA
