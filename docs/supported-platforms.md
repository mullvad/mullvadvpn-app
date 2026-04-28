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

## Desktop

Some desktop OS versions go through e2e testing every day.
These are defined in [the following workflow](../.github/workflows/desktop-e2e.yml).

### Windows

Supported versions: 10 (Version 22H2) and 11 (Version 22H2 and newer). Both x86-64 and ARM64.

#### Tested versions

| Version        | Architecture | Type of test |
|----------------|--------------|--------------|
| 10 (22H2)      | x86-64       | e2e, manual  |
| 11 (22H2)      | x86-64       | e2e, manual  |
| 11 (25H2)      | ARM64        | manual       |

### macOS

Supported versions: The three most recent major releases. Both Intel (x86-64) and Apple Silicon (ARM64).

#### Tested versions

| Version          | Architecture   | Type of test |
|------------------|----------------|--------------|
| 14 (Sonoma)      | x86-64 (Intel) | e2e          |
| 14 (Sonoma)      | ARM64          | e2e, manual  |
| 15 (Sequoia)     | ARM64          | e2e, manual  |
| 26 (Tahoe)       | ARM64          | e2e, manual  |

### Linux

Supported versions:
- **Ubuntu**: The two latest LTS releases and the latest non-LTS releases
- **Fedora**: Versions that are not yet [EOL](https://fedoraproject.org/wiki/End_of_life)
- **Debian**: 12 and newer

Both x86-64 and ARM64 are supported on all supported distributions.

On Linux, we test using the Gnome desktop environment. The app should work in other
DEs, but we don't regularly test those.

#### Tested versions

| Distribution + version | Architecture | Type of test |
|------------------------|--------------|--------------|
| Debian 12              | x86-64       | manual       |
| Debian 13              | x86-64       | e2e, manual  |
| Ubuntu 22.04           | x86-64       | e2e, manual  |
| Ubuntu 24.04           | x86-64       | e2e, manual  |
| Ubuntu 25.04           | x86-64       | e2e, manual  |
| Ubuntu 25.10           | x86-64       | e2e, manual  |
| Fedora 42              | x86-64       | e2e, manual  |
| Fedora 43              | x86-64       | e2e, manual  |

## Android

Supported versions: Android 9 and newer.
Supported targets: Mobile, TV, and tablets.
Architectures: ARM32, ARM64, x86, and x86-64.

The Android ecosystem is broad in terms of versions, flavors, and manufacturers. Our goal is to
support all versions within the version constraint. However, some manufacturer-specific flavors may
remove system components that our app relies on, which can affect functionality.

### Tested versions

| Version        | Architecture | Type of test |
|----------------|--------------|--------------|
| Android 16     | ARM64        | e2e, manual  |

Manual tests are performed on a set of devices that varies between releases.

## iOS

Supported versions: 17.0 and newer.

### Tested versions

| Version        | Type of test |
|----------------|--------------|
| 17             | manual       |
| 18             | manual       |
| 26             | e2e, manual  |
