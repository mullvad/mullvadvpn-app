# Supported platforms

These are the operating systems, versions and architectures that the app officially
supports and actively tests. It might work on many more versions, but we don't
test for those and can't guarantee the quality or security.

We do not test all supported versions equally. Exhaustively testing every OS version
for every target architecture is unrealistic as there are too many combinations.
That's why this document exists, to better explain what is actually tested, and to
what extent.

## Types of testing

- **e2e** - Automatic end-to-end integration testing. Runs extensive tests
  automatically, both nightly and before releases.
- **manual** - Thorough human testing of the app prior to making a release.

## Windows

Supported versions: 10 (Version 19H2 and newer) and 11. Both x64 and arm64.

### Tested versions

| Version        | Architecture | Type of test |
|----------------|--------------|--------------|
| 10 (21H2)      | x64          | e2e, manual  |
| 11 (22h2)      | x64          | e2e, manual  |
| 11 (25h2)      | ARM          | manual       |

## macOS

Supported versions: The three most recent major releases. Both 64-bit Intel, and ARM.

### Tested versions

| Version          | Architecture | Type of test |
|------------------|--------------|--------------|
| 14 (Sonoma)      | Intel        | e2e          |
| 14 (Sonoma)      | ARM          | e2e, manual  |
| 15 (Sequoia)     | ARM          | e2e, manual  |
| 26 (Tahoe)       | ARM          | e2e, manual  |

## Linux

Supported versions:
- **Ubuntu**: The two latest LTS releases and the latest non-LTS releases
- **Fedora**: Versions that are not yet [EOL](https://fedoraproject.org/wiki/End_of_life)
- **Debian**: 12 and newer

Both x86_64 and ARM64 is supported on all supported distributions.

On Linux, we test using the Gnome desktop environment. The app should work in other
DEs, but we don't regularly test those.

### Tested versions

| Distribution + version | Architecture | Type of test |
|------------------------|--------------|--------------|
| Debian 12              | x86_64       | manual       |
| Debian 13              | x86_64       | e2e, manual  |
| Ubuntu 22.04           | x86_64       | e2e, manual  |
| Ubuntu 24.04           | x86_64       | e2e, manual  |
| Ubuntu 25.04           | x86_64       | e2e, manual  |
| Ubuntu 25.10           | x86_64       | e2e, manual  |
| Fedora 42              | x86_64       | e2e, manual  |
| Fedora 43              | x86_64       | e2e, manual  |

## Android

Supported versions: 8 and newer.

### Tested versions

| Version        | Architecture | Type of test |
|----------------|--------------|--------------|
| ...            |              |              |

## iOS

Supported versions: 17.0 and newer.

### Tested versions

| Version        | Type of test |
|----------------|--------------|
| 17             | manual       |
| 18             | manual       |
| 26             | e2e, manual  |
