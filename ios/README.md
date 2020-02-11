# Screenshots for AppStore

The process of taking AppStore screenshots is automated using a UI Testing bundle and Snapshot tool,
a part of Fastlane tools.

## Configuration

The screenshot script uses the real account token to log in, which is provided via Xcode build 
configuration.

1. Create the build configuration using a template file:
   
   ```
   cp ios/Configurations/Screenshots.xcconfig.template ios/Configurations/Screenshots.xcconfig
   ```

1. Edit the configuration file and put your account token without quotes:
   
   ```
   vim ios/Configurations/Screenshots.xcconfig
   ```

## Prerequisitives

1. Make sure you have [rvm](https://rvm.io) installed.
1. Install Ruby 2.5.1 or later using `rvm install <VERSION>`.
1. Install necessary third-party ruby gems:
   
   ```
   cd ios
   bundle install
   ```

## Take screenshots

Run the following command to take screenshots:

```
cd ios
bundle exec fastlane snapshot
```

Once done all screenshots should be saved under `ios/Screenshots` folder.
