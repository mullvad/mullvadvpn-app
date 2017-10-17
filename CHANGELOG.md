# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).

### Categories each change fall into

* **Added**: for new features.
* **Changed**: for changes in existing functionality.
* **Deprecated**: for soon-to-be removed features.
* **Removed**: for now removed features.
* **Fixed**: for any bug fixes.
* **Security**: in case of vulnerabilities.


## [Unreleased]
### Added
- Possibility to shut down the daemon via an RPC call.


## [2017.1-beta5] - 2017-10-17
### Changed
- Upgrade the OpenVPN plugin to reduce risk of panics

### Fixed
- Change log level to reduce log file size
- Introduce minimum delay between failed VPN tunnel connections, to reduce load on the computer in special cases

### Security
- Authenticate RPC connections towards the backend
- Reject revoked server certificates

## [2017.1-beta4] - 2017-10-05
Nothing of interest

## [2017.1-beta3] - 2017-10-05
### Added
- Automatically secure connection on login

### Changed
- Upgrade JSON-RPC library for more stable communication to our account server
- Remove the auto-secure setting
- Show the destination country while securing the connection
- Clean up the server list

### Fixed
- No longer clear the account token input field when navigating to and from the settings
- Show the main UI window on start when the user is not logged in

## [2017.1-beta2] - 2017-09-27
### Added
- Support for removing the account number from the CLI.

### Changed
- Improved logging in the frontend in case of backend communication failure.

### Fixed
- Fix logout bug not removing the account number correctly.
- Don't show city and country in the frontend when tunnel is not connected.
- Don't try to automatically establish a tunnel from the frontend if the login failed.


## [2017.1-beta1] - 2017-09-27
### Added
- Initial closed beta release. Can set up a tunnel and protect against leaks on macOS.
