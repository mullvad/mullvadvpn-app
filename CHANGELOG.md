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
- "Allow LAN" setting that configures if the app should allow communication to the LAN (private
  networks: 10/8, 192.168/16 and 172.16/12) while the app is in the secured state.

### Changed
- The tray icon now indicates wether the app is allowing traffic outside the tunnel or not. If the
  app blocks traffic because the tunnel is not connected the tray icon will indicate this with a
  green lock with a red dot.
- While connecting, a message telling the user that internet accesss is blocked is shown.


## [2017.1-beta7] - 2017-12-13
### Added
- Buffer size and fast-io parameters to OpenVPN. Can double the speed on high latency connections.
- Download a list of our current servers on startup, instead of having a bundled list of servers in
  the app that does not receive updates.
- Backup account numbers in a file so that they are harder to lose.
- Include the OpenVPN log in the problem report. IP addresses and MAC addresses are redacted before
  the logs are sent.

### Fixed
- Hold off notifying the frontend of the 'unsecure' state until the VPN tunnel is actually
  completely disconnected.
- Show the VPN GUI on all macOS workspaces, not only the one where it was started.

### Changed
- Renamed daemon binary from `mullvadd` to `mullvad-daemon`.

### Security
- DNS leak found when using redirect firewall rules and a custom DNS forwarder. Replaced all of that
  with strict DNS blocking firewall rules and SystemConfiguration integration where DNS settings are
  injected to the operating system settings and constantly monitored for external changes.


## [2017.1-beta6] - 2017-11-09
### Added
- Possibility to shut down the daemon via an RPC call.
- Problem reports, for collecting and sending app logs to Mullvad support. This is fully opt-in
  and must be triggered by the user.
- Possibility to change between UDP and TCP as well as select port for OpenVPN to use.
- Possibility to copy the account number in the field where it's displayed in the GUI.

### Changed
- Escape shell arguments better in both backend daemon and GUI.
- Rename the macOS PF firewall anchor created by the program to "mullvad".
- Change settings format from toml to json. To enable storing more advanced settings types.

### Fixed
- Shut down the backend daemon when quitting the app from the GUI. It was previously kept alive.
- Sign the macOS binaries with SHA1 in addition to SHA256. Enables running on 10.9 and 10.10.


## [2017.1-beta5] - 2017-10-17
### Changed
- Upgrade the OpenVPN plugin to reduce risk of panics

### Fixed
- Change log level to reduce log file size
- Introduce minimum delay between failed VPN tunnel connections, to reduce load on the computer in
  special cases

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
