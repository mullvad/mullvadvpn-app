# Mullvad VPN iOS app

This is the iOS version of the Mullvad VPN app. The app can be found on the Apple [App Store].

All releases have signed git tags on the format `ios/<version>`. For changes between each
release, see the [changelog].

For Xcode project configuration please refer to [Configure Xcode project] section of build 
instructions document.

[App Store]: https://apps.apple.com/us/app/mullvad-vpn/id1488466513
[changelog]: CHANGELOG.md
[Configure Xcode project]: BuildInstructions.md#configure-xcode-project

## Code formatting

The codebase is formatted using [SwiftFormat](https://github.com/nicklockwood/SwiftFormat). Please 
format all contributions using the latest version of formatter.

```
swiftformat ios/
```

Install the latest version of SwiftFormat via Homebrew:

```
brew install swiftformat
```

CI uses the latest version available on Homebrew to check formatting, so please keep your local 
installation up to date, if you see it complain:

```
brew upgrade swiftformat 
```

## Screenshots for AppStore

The process of taking AppStore screenshots is automated using a UI Testing bundle and Snapshot tool,
a part of Fastlane tools.

### Configuration

The screenshot script uses the real account number to log in, which is provided via Xcode build 
configuration.

1. Create the build configuration using a template file:
   
   ```
   cp ios/Configurations/Screenshots.xcconfig.template ios/Configurations/Screenshots.xcconfig
   ```

1. Edit the configuration file and put your account number without quotes:
   
   ```
   vim ios/Configurations/Screenshots.xcconfig
   ```

### Prerequisitives

1. Make sure you have [rvm](https://rvm.io) installed.
1. Install Ruby 2.5.1 or later using `rvm install <VERSION>`.
1. Install necessary third-party ruby gems:
   
   ```
   cd ios
   bundle install
   ```

### Take screenshots

Run the following command to take screenshots:

```
cd ios
bundle exec fastlane snapshot
```

Once done all screenshots should be saved under `ios/Screenshots` folder.

### Localizations

The iOS app does not yet have translations.

There was a script in the past to help with translations, but it was removed.
Whenever we want to start adding translations for real, this script can be
resurrected from the git history if we deem it to be the best path forward.
Look for `ios/requirements.txt`.

## Icons and assets

For a general overview of icons and graphics read [the graphics readme](../graphics/README.md).

To copy graphical assets from the desktop GUI and generate iOS assets, run:
```bash
ios/convert-assets.rb --app-icon
ios/convert-assets.rb --import-desktop-assets
ios/convert-assets.rb --additional-assets
```

## Cached relays

The script `relays-prebuild.sh` runs on each Xcode build and will download and cache a list of relays if it is not already present for a given configuration.
The cached list for a given configuration will always override the current relays file.
To get a fresh relay file on demand, issue a `clean` command to Xcode and re-build the project.
