# Changelog
All changes to the software that can be noticed from the users' perspective should have an entry in
this file. Except very minor things that will not affect functionality, such as log message changes
and minor GUI adjustments.

### Format

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).

Entries should have the imperative form, just like commit messages. Start each entry with words like
add, fix, increase, force etc.. Not added, fixed, increased, forced etc.

Line wrap the file at 100 chars.                                              That is over here -> |

### Categories each change fall into

* **Added**: for new features.
* **Changed**: for changes in existing functionality.
* **Deprecated**: for soon-to-be removed features.
* **Removed**: for now removed features.
* **Fixed**: for any bug fixes.
* **Security**: in case of vulnerabilities.


## [Unreleased]
### Added
- Add option to enable or disable IPv6 on the tunnel interface.

### Changed
- The "Buy more credit" button is changed to open a dedicated account login page instead of one
  having a create account form first.


## [2018.2-beta3] - 2018-08-09
### Added
- Create a new UI log file for every UI execution session, and preserve the log from the previous
  session.
- Account token can be copied to the clipboard by clicking on it in the account settings screen.
- Automatically scroll to selected country/city in locations view.
- Show system notifications when connection state changes and the window is not visible.
- Add launch view displayed when connecting to system service.

### Changed
- Format the expiry date and time using the system locale.
- Account tokens are now required to have at least ten digits.

#### macOS
- Rename directores for settings, logs and cache from `mullvad-daemon` to `mullvad-vpn`.

#### Windows
- Use local user directory to store system service settings and GUI electron cache, instead of the
  roaming user directory.
- Where the system service would use `%LOCALAPPDATA%\Mullvad\Mullvad VPN\` it now just uses
  `%LOCALAPPDATA%\Mullvad VPN\`

### Fixed
- Ignore empty strings as redaction requests in the problem report tool, to avoid adding redacted
  markers between every character of the log message.
- Previously logged in users won't be going through login view when restarting the app, instead
  will be taken straight to main view.


## [2018.2-beta2] - 2018-07-18
### Added
- Bundle the root CA signing the API and only trust that single one, limiting
  trust to a single root CA
- Add a unique UUID to problem reports. Makes it easier for Mullvad support staff to find reports.
- Add "auto-connect" setting in daemon, and make it configurable from CLI. Determines if the daemon
  should secure the network and start establishing a tunnel directly when it starts on boot.
- Add "auto-connect" and "auto-start" options to the application preferences view.

#### Windows
- Include version information (meta data) in executables and DLLs.
- Include manifest in daemon so it always runs with administrator privileges.
- Add sidebar graphic in installer/uninstaller.

### Changed
- App now uses statically linked OpenSSL on all platforms.
- Add OpenVPN logs at the top of the problem report instead of middle, to aid support work.
- Lower per log size limit in the problem report to 128 kiB.
- Relay list is now updated periodically automatically, not only when the daemon starts.

#### Windows
- Rename tunnel interface to "Mullvad".
- Change tunnel interface metric for both IPv4 and IPv6.

### Fixed
- Disable account input when logging in.
- Keep the user input in problem report form while the app runs, or until the report is successfully
  submitted.

#### Windows
- Hide the app icon from taskbar.
- Autohide the main window on focus loss.
- Loosen up firewall rules to allow incoming requests on tunnel interface.
- Properly stop the service, announcing errors to the system, in the event of initialization or
  runtime error.


## [2018.2-beta1] - 2018-07-02
### Added
- Refresh account expiration when account view becomes visible.
- Add `tunnel` subcommand to manage tunnel specific options in the CLI.
- Add support for passing the `--mssfix` argument to OpenVPN tunnels.
- Add details to mullvad CLI interface error for when it doesn't trust the RPC file.
- Include the last two OpenVPN logs in problem reports instead of only the last.
- Prevent two instances of the daemon to run at the same time.
- Add CLI command for fetching latest app versions and verifies whether the running version is
  supported.
- Add `version` subcommand in the CLI to show information about current versions.
- Add a flag to daemon to print log entries to standard output without timestamps.
- Filter out and ignore DNS lookup results for api.mullvad.net that are bogus (private etc.)
- Bundle the Mullvad API IP address with the app and introduce a disk cache fallback method for
  when DNS resolution fails.
- Automatic rotation of the daemon log. The existing log is renamed to `daemon.old.log` on daemon
  startup.
- Add `status listen` subcommand in the CLI to continuously monitor the tunnel state.
- Log errors present in initialization sequence to the log file.

#### macOS
- Add colors to terminal output.
- Warn if daemon is running as a non-root user.
- Make the pkg installer uninstall any `<=2018.1` version of the app before installing itself.

### Changed
- Changed "Contact support" label to "Report a problem" in settings menu
- Change all occurrences of "MullvadVPN" into "Mullvad VPN", this affects
  paths and window captions etc.
- Improve account token hint to be the same length as an expected token.
- Update `problem-report` binary to automatically collect log files in predefined known Mullvad log
  directories.
- Replaced previously bundled OpenVPN 2.4.4 with statically linked 2.4.6 version containing
  Mullvad patches for faster connect and other improvements.
- Increase the OpenVPN receive and send buffers from 524288 to 1048576 bytes (1MiB).
- Make the log, cache, settings and RPC address directories configurable via the following
  environment variables: `MULLVAD_LOG_DIR`, `MULLVAD_CACHE_DIR`, `MULLVAD_SETTINGS_DIR` and
  `MULLVAD_RPC_ADDRESS_PATH`.

#### macOS
- The installer changed from dmg to pkg format.
- The daemon is installed as a launchd daemon and started on install and on boot.
- Move daemon logs to `/var/log/mullvad-daemon/`, settings to `/etc/mullvad-daemon/` and cache to
  `/var/root/Library/Caches/mullvad-daemon/`.

### Removed
- Remove the `shutdown` command from the CLI.

### Fixed
- Fix scroll flickering.
- Fix bug in account input field that advanced the cursor to the end regardless its prior position.
- Redact all 16 digit numbers from problem report logs. Extra safety against accidentally sending
  account numbers.
- Fix OpenVPN plugin search directory to be the installation directory.
- Reduce RPC timeout to Mullvad API server.
- Fix OpenVPN warning about usage of AES-256-CBC cipher.
- Fix "Out of time" screen status icon position.
- If necessary, create parent directories for RPC connection info file and tunnel log.
- Fix error message when attempting to login when the daemon isn't running .


## [2018.1] - 2018-03-01
### Changed
- Redact all account numbers in the account number history from problem reports instead of only the
  currently logged in one.

### Fixed
- Increase a timeout for problem report collection to fix a timeout error on slower machines.
- Fix a memory leak in the problem report collection routine.
- Fix an issue when viewing a problem report brought up a dialog to choose the application to open
  the file.


## [2018.1-beta10] - 2018-02-13
### Added
- Show the app version in the settings view.

### Changed
- Require confirmation when sending problem reports without an email address.

### Fixed
- Fix erroneous styles in the settings view.

### Security
- Update the CRL with newly revoked server certificates.


## [2018.1-beta9] - 2018-01-30
### Added
- Uses the https://am.i.mullvad.net/ service to figure out location and public IP of the device.
  The app then shows this information in the unsecured state.
- Argument to the daemon, `--resource-dir <path>`, that allows customizing where it will look for
  needed resource files.
- A very stylish map now indicates where you are connecting through.

### Fixed
- Fixed a bug where the problem report tool would redact some things in the logs which were not
  IPv6 addresses, but looked like ones.
- Show a better error message when api.mullvad.net is unreachable.
- Fix bug getting daemon state on frontend start instead of assuming it.


## [2018.1-beta8] - 2018-01-09
### Added
- "Allow LAN" setting that configures if the app should allow communication to the LAN (private
  networks: 10/8, 192.168/16 and 172.16/12) while the app is in the secured state.
- The app can now be used to connect to all our servers rather than a smaller subset. The list
  of servers is automatically updated when the app starts.
- The location selector now shows if the country or city has any active servers.

### Changed
- The tray icon now indicates whether the app is allowing traffic outside the tunnel or not. If the
  app blocks traffic because the tunnel is not connected the tray icon will indicate this with a
  green lock with a red dot.
- While connecting, a message telling the user that internet accesss is blocked is shown.
- Default to selecting servers in Sweden to increase the likelyhood of a fast and stable connection.
- Scrollbars will automatically hide when not scrolling.

### Removed
- Remove the unsafe Camellia and Seed ciphers from the list of allowed OpenVPN ciphers.


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
