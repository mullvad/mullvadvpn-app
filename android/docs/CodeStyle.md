# Code style

This document describes the code style of the Android app and how to format Kotlin and XML files,
which is checked in CI. The conventions that are common to all Mullvad projects can be found in the
[Mullvad coding guidelines](https://github.com/mullvad/coding-guidelines).

For static analysis (Detekt, Android Lint and Konsist), see the [test suite overview](TestSuite.md).

## Kotlin formatting

[`ktfmt`](https://github.com/facebook/ktfmt) is used for Kotlin formatting. It runs through the
[`ktfmt` Gradle plugin](https://github.com/cortinico/ktfmt-gradle), which is configured in
`build.gradle.kts`.

Command to format:
```bash
./gradlew ktfmtFormat :build-logic:ktfmtFormat
```

Command to check the formatting without changing any files (this is what CI runs):
```bash
./gradlew ktfmtCheck :build-logic:ktfmtCheck
```

See the [official documentation](https://github.com/facebook/ktfmt) for how to use `ktfmt` as the
default formatter in Android Studio. Make sure to set `kotlinLangStyle` as "Code Style" and to set
the project to rely on the `EditorConfig` (`.editorconfig` file).

## XML formatting

XML files are formatted with the script `scripts/tidy.sh`, which is a helper script for running the
tool [`tidy`](https://github.com/htacg/tidy-html5). `tidy` needs to be installed unless the
container image is used.

Command to format:
```bash
scripts/tidy.sh format
```

Command to format and check for any changes (this is what CI runs):
```bash
scripts/tidy.sh formatAndCheckDiff
```

### macOS

The script relies on GNU `sed`. With the `sed` that ships with macOS it creates a large number of
files ending with `-e`. To fix this, install GNU `sed` with `brew install gnu-sed` and add it to
your `PATH` so that it's used instead of the macOS one:
`PATH="$HOMEBREW_PREFIX/opt/gnu-sed/libexec/gnubin:$PATH"`.
