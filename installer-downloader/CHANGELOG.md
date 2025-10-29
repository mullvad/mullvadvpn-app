# Changelog
All notable changes are recorded here.

### Format

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).

Entries should have the imperative form, just like commit messages. Start each entry with words like
add, fix, increase, force etc.. Not added, fixed, increased, forced etc.

Line wrap the file at 100 chars.                                              That is over here -> |

### Categories each change falls into

* **Added**: for new features.
* **Changed**: for changes in existing functionality.
* **Deprecated**: for soon-to-be removed features.
* **Removed**: for now removed features.
* **Fixed**: for any bug fixes.
* **Security**: in case of vulnerabilities.

## [Unreleased]

## [1.2.0] - 2025-09-30
### Changed
- Ignore releases with a rollout of `0`.


## [1.1.0] - 2025-06-16
### Added
#### Windows
- Add support for installing previously downloaded apps when internet access is unavailable.

### Fixed
- Fix downloads hanging indefinitely on switching networks
#### macOS
- Fix rendering issues on old (unsupported) macOS versions.

## [1.0.0] - 2025-05-13
### Fixed
#### Windows
- Use default font instead of Segoe Fluent Icons for back arrow.


## [0.3.0] - 2025-05-06
### Added
#### Windows
- Add window icon.

### Changed
- Changed title to "Mullvad VPN loader".


## [0.2.0] - 2025-04-08
- Initial support for downloading and installing the latest version of the Mullvad VPN app on
  Windows and macOS.
