## Gemfiles for uploading to Appstore Connect.
These gemfiles specify the `fastlane` dependencies needed to run our script that uploads our iOS app
to the AppStore. This should be done once when setting up the upload VM.

To set up fastlane, you should invoke `bundle` like so:
```bash
bundle install
```

To run fastlane, one should use:
```bash
bundle exec fastlane $fastlane_command
```

To update fastlane dependencies, one should run `bundle update` on a local machine, verify that the
changes are sound, and update the upload VM accordingly if they are.

