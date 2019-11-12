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
#### Windows
- Full WireGuard support, GUI and CLI.
- Install Wintun driver that provides the WireGuard TUN adapter.
- Remove Mullvad TAP adapter on uninstall. Also remove the TAP driver if there are no other TAP
  adapters in the system.

### Changed
- Notifications shown when connecting to a server include its location.

#### Windows
- Use a larger icon in notifications on Windows 10.

### Fixed
#### Windows
- Detect removal of the OpenVPN TAP adapter on reconnection attempts.
- Improve robustness in path environment variable logic in Windows installer. Handle the case
  where the registry value type is incorrectly set to be a regular string rather than an expandable
  string.


## [2019.10-beta1] - 2019-11-06
This release is for Android only.

### Added
#### Android
- Use authenticated URLs to go to wireguard key page on website.
- WireGuard key fragment has been made more similar to its desktop counterpart.

### Fixed
- Fix bad file descriptor errors caused by sending a file descriptor between the daemon and the
  `wireguard-go` library.
- Recreate tun device after a fixed number of connection attempts on the same tun device. Breaks
  infinite reconnection loops on broken tun devices.


## [2019.9] - 2019-10-11
### Added
- Add ability to submit vouchers from the CLI.

#### Linux
- Add a symlink for `mullvad-problem-report` directly in `/usr/bin`. So the tool is available.

#### Windows
- Install the OpenVPN certificate to avoid the TAP adapter driver installation warning on
  Windows 8 and newer.

### Changed
#### Windows
- Rename the `problem-report` tool to `mullvad-problem-report`.

### Fixed
- Fix Norwegian (Bokmal) language detection.
- Fix missing localizations when formatting date and time in Norwegian (Bokmal).
- Use authenticated URL to go to account page from expired account view.

#### macOS
- Remove `mullvad` and `mullvad-problem-report` symlinks from `/usr/local/bin` on uninstall.


## [2019.9-beta1] - 2019-10-08
### Added
- Add ability to change the desktop GUI language from within Settings.
- Add ability to create new accounts from the CLI.

#### Windows
- Add CLI tools (the resource/ directory) to the system PATH.

#### macOS
- Notarize release builds with Apple. Making them run without warning on 10.15 Catalina.

#### Android
- Add settings button in launch and login screens. Making it possible to reach the problem report.
- Add support for Android 5.x Lollipop.
- Allow logging in without connectivity.

### Changed
- Account and WireGuard keys links in the App will now log the user in automatically.
- Update FAQ URL to `https://mullvad.net/help/tag/mullvad-app/`.

### Removed
- Remove support for `MULLVAD_LOCALE` environment variable.

#### Android
- Remove connect action button in notification when logged out.

### Fixed
- Fix `mullvad relay update` to trigger a relay list download even if the existing cache is new.
- Don't include problem-report arguments in error logging. Stops user email from ending up in the
  log file on error.
- Fix handling of tunnel file descriptor for WireGuard. Duplicating and closing it correctly.

#### Android
- Show WireGuard key age in local timezone instead of UTC.
- Android 6 and older: Fix notification button icons.
- Fix collapsing tunnel information causing tunnel out IP address information to be lost.
- Various stability fixes.

#### Windows
- More adjustments in online/offline detection logic. Should prevent more users from being stuck
  in the offline state. Should also make the app notice network disconnects faster.


## [2019.8] - 2019-09-23
This release is identical to 2019.8-beta1


## [2019.8-beta1] - 2019-09-19
### Added
- Add ability to replace the WireGuard key with a new one. Allows manual key rotation.
- Show age of currently set WireGuard key.
- Add bridge selection under "Select location" view, when the bridge mode is set to "On".

#### Android
- Initial support for the Android platform.

### Changed
- Decreased default MTU for WireGuard to 1380 to improve performance over 4G
- WireGuard key page now shows a label explaining why buttons are disabled when in a blocked state
- WireGuard key generation will try to replace old key if one exists.
- Show banner about new app versions only if current platform has changes in latest release.
- Don't make a GeoIP lookup by default in CLI status command. Add --location flag for enabling it.
- Sort relay locations and hostnames with natural sorting. Meaning `se10` will show up after `se2`.
- Show inactive relays as disabled instead of hiding them completely from location selection list.
- Upgrade Electron from version 4 to version 6.

#### Windows
- Change uninstaller registry key name from `Mullvad VPN` to a generated GUID.

### Fixed
- Fix old settings deserialization to allow migrating settings from versions older than 2019.6.
- Fix various small issues in GUI<->daemon communication.
- Make GUI WireGuard key verification resilient to failure.
- Fix issue where daemon would try and connect with UDP when the tunnel protocol is set to OpenVPN
  and the bridge mode is set to "On".
- Don't start ping monitor loop if first ping fails when checking WireGuard connection.
- Respect localization when sorting the relay locations list.

#### macOS
- Unregister the app properly from the OS when running the bundled `uninstall.sh` script.

#### Linux
- Fix bug in netlink parsing in offline detection code.

### Removed
#### Windows
- Removed logic that implemented monitoring and enforcement of DNS settings.


## [2019.7] - 2019-08-12
### Added
- Add more details to the block reason shown in GUI when the daemon fails to generate tunnel
  parameters.

### Fixed
- Check and adjust relay and bridge constraints when they are updated, so no incompatible
  combinations are used.
- Fix panic when running CLI "mullvad relay set custom" without any more arguments.


## [2019.7-beta1] - 2019-08-08
### Added
- Add new settings page for generating and verifying wireguard keys.
- Automatically generate and upload WireGuard keys on Linux and macOS.
- Allow activating and using WireGuard from the GUI under advanced settings on Linux and macOS.
- Add `factory-reset` CLI command for removing settings, logs and clearing the cache.

### Changed
- Upgrade OpenVPN from 2.4.6 to 2.4.7.
- Upgrade OpenSSL from 1.1.0h to 1.1.1c.
- Upgrade wireguard-go library to v0.0.20190805.

### Fixed
- Mark CLI `bridge set state` argument as required to avoid a crash.
- The VPN service on Windows will now be restarted when it crashes.
- Retry to connect when WireGuard tunnel fails due to a bad file descriptor.

#### Linux
- Improve resolv.conf based DNS management to detect changes to file.


## [2019.6] - 2019-07-15
### Added
- Add simplified Chinese translations.


## [2019.6-beta1] - 2019-07-08
### Added
- Add a switch to turn off system notifications under Preferences in the GUI.

#### Windows
- Add migration logic to restore lost settings after major Windows update.

#### macOS
- Add the Mullvad CLI frontend and problem report CLI tool to the PATH, so it can be
  run directly from a terminal.

### Fixed
- Fix the mix of traditional and simplified Chinese. Separating them to two locales and fall back
  to English where translations are missing.

#### Windows
- Adjust network interface checks in offline detection logic. Prevents the app from being stuck
  in the offline state when the computer is in fact online.

#### Linux
- Fix some netlink packet parsing error in DNS handling.
- Improve offline check so if it fails, it always fails as online.


## [2019.5] - 2019-06-17
### Added
- Add Norwegian translations.


## [2019.5-beta1] - 2019-06-13
### Added
- Add support for roaming between connections when using wireguard.
- Allow mDNS/discover to 239.255.255.251 when local network sharing is enabled. This change fixes
  the Wi-Fi calling via iPhone when both devices are on the same network.
- Allow incoming DHCPv4 requests and outgoing responses if allow local network is enabled. Enables
  being a DHCPv4 server.
- Add GUI translations for Italian, Japanese, Dutch, Portugese, Russian and Turkish.
- Add missing GUI translations for Czech Republic, USA and UK in the select location view.
- Add translations for the current location displayed on the main screen in the GUI.
- Allow a subset of NDP (Router solicitation, router advertisement and redirects) in the firewall.
- Add automatic Shadowsocks bridge usage. Will automatically try to obfuscate the tunnel via
  Shadowsocks after a number of failed connection attempts.
- Automatically include frontend logs in problem report when ran from CLI.

#### Linux
- Add standard window decorations to the application window.

### Changed
- Relax the allow local network rules slightly. only checking either source or destination IP field
  instead of both. They are still unroutable.
- CLI commands that are just intermediate commands, and require another level of subcommands, will
  automatically print the available subcommands, instead of an error if none is given.

### Removed
- Remove the `help` subcommand in the CLI. Instead get help with the `--help` long flag.

### Fixed
- Stop allowing the wrong IPv6 net fe02::/16 in the firewall when allow local network was enabled.
  Instead allow the correct multicast nets ff02::/16 and ff05::/16.
- Fix the regression that allowed to get past the login screen using the invalid account token.
- Fix the GUI crash caused by a derefence of the already released remote object in Electron.

#### macOS
- Raise max number of open files for the daemon to 1024. Should prevent threads from panicking.
- Fix the visual defect that resulted in a semi-transparent grey line rendered above the window.

#### Windows
- Add better offline detection.

#### Linux
- Fix `systemd-resolved` detection by better checking `/etc/resolv.conf` symlinks.
- Improve detection of whether NetworkManager is the preferred DNS solution.


## [2019.4] - 2019-05-08
This release is identical to 2019.4-beta1


## [2019.4-beta1] - 2019-05-02
### Added
- When IPv6 is enabled, get both exit IP versions from am.i.mullvad.net and show.
- Add translations for country and city names in the relay list and map.

### Fixed
- Reset the tray icon padlock to the unsecured state when losing connectivity with the daemon.

#### Windows
- Increase timeout when updating DNS settings. Should make the DNS management fail less often.
- Use dynamic naming of TAP adapter to avoid collisions with existing adapters.
- On Windows Surface devices the keyboard now shows up correctly when selecting the account
  token input field.

### Security
#### Windows
- Make the firewall rules permanent until reboot, or until the daemon removes them. Should make
  the kill switch active even if the daemon dies unexpectedly.


## [2019.3] - 2019-04-02
### Fixed
#### Windows
- Correct dependencies on installer logger plugin to resolve installation issues on Windows 7/8.


## [2019.2] - 2019-03-28
### Removed
- Remove the Mullvad OpenVPN intermediate transition CA. Used when transitioning from the old to
  the new root CA. Now the app only bundles and trust the new Mullvad root CA valid until 2028.

### Fixed
- Read the relay list from cache only if it's newer than the version bundled in the app.


## [2019.2-beta1] - 2019-03-21
### Added
- Integrate initial Shadowsocks proxy support. Accessible via CLI.
- Add initial Wireguard support on macOS and Linux. Accessible via CLI.
- Improve "Out of time" view button leading to the account website by unlocking internet access
  before opening the browser
- Add translations for German, Spanish, French, Swedish, Chinese languages

### Fixed
- Fix the potential reconnect loop in GUI, triggered by the timeout when receiving
  the initial state of the daemon.
- Fix the bug which caused the account token history to remain stale after logout.
- Fix some notifications not appearing depending on how the window is shown and hidden while the
  tunnel state changes.
- Fix DNS when using IPv6.
- Fix the bug when the "Out of time" view remained visible, even when the app managed to reconnect
  the VPN tunnel after a successful credit top-up.
- Sort the relay location list alphabetically in the GUI.

#### Linux
- Fix startup failure when network device with a hardware address that's not a MAC address is
  present.

#### Windows
- Improve error handling related to DNS management at the time of establishing the tunnel.

### Changed
- Increase the timeout to the Mullvad API from 5 to 10 seconds.

#### Linux
- Increase `NetworkManager` DBus RPC timeout from 1 second to 3 seconds.
- Improve notification look by adding application name and icon.


## [2019.1] - 2019-01-29
This release is identical to 2019.1-beta1


## [2019.1-beta1] - 2019-01-25
### Added
- Handle "block when disconnected" extra kill-switch level in the GUI, showing the disconnected
  state as blocked when appropriate and also having a toggle switch for the setting in the Advanced
  Settings screen.
- Add a drop-down warning to notify the user when the account credits are running low.
- Allow the 169.254.0.0/16 private network in addition to the other networks allowed when local
  network sharing is enabled.
- Improve the confirmation dialog when submitting a bug report without an email specified.

#### macOS
- Add a monochromatic tray icon option for the GUI.

#### Linux
- Detect if the computer is offline. If so, don't sit in a reconnect loop, instead block and show
  an error message.
- Add a toggle switch to allow the app to start minimized on Linux, so that only the tray icon is
  initially visible.

### Changed
- Disable buttons and menus that open external links when the app knows that there is no internet
  connection.
- The auto-start and auto-connect toggles in the GUI have been reworked so that auto-connect
  configures the GUI to automatically connect when it starts and so that it will only auto-connect
  on boot when both settings are set.

### Fixed
- Stop GUI from glitching during the short reconnect state.
- Dismiss notifications automatically after four seconds in all platforms.
- Fix error printed from the CLI when issuing `relay update`.
- Fix relay list update interval. Should now handle sleep better.
- Prevent GUI from sending connect commands to the daemon every time it establishes a connection to
  it. Only send connect once (if auto-connect is enabled.)
- Prevent possible reconnect loop where the GUI would indefinitely reconnect to the daemon.
- Fix the bug which enabled users to return to the login view if they went to settings while
  logging in.
- Handle in the GUI, if something external changes the account token in the daemon. I.e. triggered
  by CLI unsetting or resetting it.

#### Linux
- Fix Debian package not upgrading properly due to a bug in the post-remove script.
- Wait for NetworkManager and systemd-resolved services to start before daemon starts on platforms
  with systemd and those two services. Prevents the daemon from using the wrong DNS API.

#### Windows
- Gracefully block when TAP adapter is missing or disabled, instead of retrying to connect.

### Security
#### Linux
- Poll netfilter to verify firewall rules were added correctly. On Ubuntu 14.04 netfilter did not
  return any error, but it also ignored the rules the daemon tried to add.


## [2018.6] - 2018-12-12
This release is identical to 2018.6-beta1


## [2018.6-beta1] - 2018-12-05
### Added
- CLI command `relay update` that triggers an update of the relay list in the daemon.
- Add extra level of kill-switch called "block when disconnected". Blocks all network traffic even
  in the disconnected state. Not activated by default and can be changed via the CLI subcommand
  `block-when-disconnected`.

#### macOS
- Detect if the computer is offline. If so, don't sit in a reconnect loop, instead block and show
  an error message.
- Add ability to debug firewall rules with the `TALPID_FIREWALL_DEBUG` variable.

#### Windows
- Install tray icon in visible part of the notification area.

### Changed
- Split DNS management from Firewall management to allow restoring DNS earlier and showing more
  detailed errors to users.

### Fixed
- Cancel pending system notifications when the app becomes visible.
- Transition to connected state after all routes are configured. Avoids problems with reaching the
  internet directly after the app says it's connected.
- Disable keep alive on API RPC requests. Should stop reuse of invalid sockets after tunnel state
  changes.

#### macOS
- Fix permissions on log dir so problem-report tool has permission to read daemon logs.

#### Windows
- Use proper app id in the registry. This avoids false-positives with certain anti-virus software.
- Handle sleep/resume events to quickly restore the tunnel when the machine wakes up.
- Add default route to fix NLA issues (Microsoft Store/Office/etc say the machine is offline).
- Update installer to not rely on WMI when enumerating network adapters.
- Increase timeout waiting for OpenVPN to shut down cleanly.
- Sign the bundled openvpn.exe binary. Should make some anti-virus software complain less.


## [2018.5] - 2018-11-15
### Changed
- Replace OpenVPN root CA certificate bundled with the app to the new Mullvad root CA.

### Fixed
#### Linux
- Improve packaging on RPM based distros by re-enabling the daemon after an upgrade


## [2018.5-beta1] - 2018-11-12
### Added
- Fall back and try to connect over TCP port 443 if protocol is set to automatic and two attempts
  with UDP fail in a row. If that also fails, alternate between UDP and TCP with random ports.
- Add new system and in-app notifications to inform the user when the app becomes outdated,
  unsupported or may have security issues.
- Allow the user to view the relay in/out IP address in the GUI.
- Add OpenVPN proxy support via CLI.
- Allow DHCPv6 in the firewall.

### Fixed
- Pick new random relay for each reconnect attempt instead of just retrying with the same one.
- Make the `problem-report` tool fall back to the bundled API IP if DNS resolution fails.

#### macOS
- Correctly backup and restore search domains and other DNS settings.

#### Linux
- Disable GPU acceleration on Linux to fix App on Ubuntu 14.04 and other older distributions.
- Improve DNS management detection. Evaluates which way the system handles DNS before each new
  VPN tunnel is established instead of only on computer boot.
- Set DNS search domain when using the systemd-resolved. Makes it work on Ubuntu 18.10.

#### Windows
- Fix crash on Windows 7 when closing installer.

### Security
#### Linux
- Block all traffic to DNS servers other than the correct one in the tunnel. Stops potential DNS
  leaks when "Local network sharing" was enabled and DNS management failed.


## [2018.4] - 2018-10-16
### Fixed
- Fix so changing the OpenVPN mssfix setting triggers setting up a new tunnel with the new setting.


## [2018.4-beta3] - 2018-10-12
### Fixed
- Place Mssfix setting inside scrollable area.
- Fix so mssfix can be unset. Previously emptying the textbox did nothing.

#### Linux
- The app will have its window resized correctly when display scaling settings are changed. This
  should also fix bad window behaviour on startup.
- Fixed systemd-resolved DNS management. Skip using it as the DNS manager if it's running in
  consumer mode.


## [2018.4-beta2] - 2018-10-08
### Added
- Allow configuration of OpenVPN mssfix option with GUI (under Advanced Settings).

#### Windows
- Monitor and enforce IPv6 DNS settings on network interfaces (previously IPv4-only).

#### Linux
- Add support for DNS configuration using systemd-resolved and NetworkManager.

### Changed
- Auto-hide scrollbars on macOS only, leaving them visible on other platforms.
- Instead of showing the public IP of the device in the UI, we show the hostname of the VPN server
  the app is connected to. Or nothing if not connected anywhere.
- Passing `--connect-timeout 30` to OpenVPN to decrease the time the daemon
  will wait until it tries to reconnect again in the case of a broken TCP connection.
- Increase timeout parameter to OpenVPN from 15 to 20 seconds. Should make active VPN tunnels drop
  less frequent when on unstable networks.
- Reduce the transparency of "blocking internet" banner to increase the text readability.
- Make the quit button visible without needing to scroll down in the settings view.

#### Linux
- Move CLI binary to `/usr/bin/` as to have the CLI binary in the user's `PATH` by default.

### Removed
- Remove `--comp-lzo` argument to OpenVPN. Disables any possibility of establishing a VPN tunnel
  with compression.

### Fixed
#### Windows
- Use different method for identifying network interfaces during installation. Should solve some
  installation errors.
- Properly restore DNS settings on network interfaces. Fixes issue #352.


## [2018.4-beta1] - 2018-10-01
### Added
- Allow packets to the fe80::/10 and fe02::/16 IPv6 networks when local network sharing is enabled.
  Should allow IPv6 over the LAN, and mDNS host discovery which in turn should allow Apple AirDrop
  and Handover among other IPv6 based LAN discovery services.

#### Linux
- Add support for DNS configuration using resolvconf.

### Changed
- Logging in no longer requires a connection with the Mullvad API server.
- Replace repeated `Disconnecting` followed by `Connecting` notifications with a single
  `Reconnecting` notification.

### Fixed
- Don't temporarily show the unsecured state in the GUI when the app is reconnecting or blocking.
- Periodically update list of relays in the GUI.
- Redact IPv6 address that start or end with double colons in problem reports.
- Improve tray icon response time by disabling the double click handling.

### Security
- Prevent Electron from executing/navigating to files being drag-and-dropped onto the app GUI. This
  fixes [MUL-01-001](./audits/2018-09-24-assured-cure53.md#miscellaneous-issues)


## [2018.3] - 2018-09-17
### Changed
#### macOS
- Move the CLI binary (`mullvad`) back into the `Resources/` directory. A bug caused the app to not
  be signed if it was placed in the app root directory.

### Security
#### Windows
- Lock the installation directory to `C:\Program Files\Mullvad VPN`. This prevents potential local
  privilege escalation by ensuring all binaries executed by the `SYSTEM` user, as part of the
  Mullvad system service, are stored where unprivileged users can't modify them. This fixes
  [MUL-01-004](./audits/2018-09-24-assured-cure53.md#identified-vulnerabilities).


## [2018.3-beta1] - 2018-09-13
### Added
- Add option to enable or disable IPv6 on the tunnel interface. It's disabled by default.
- Log panics in the daemon to the log file.
- Warn in the Settings screen if a new version is available.
- Add a "blocked" state in the app that blocks the entire network and shows a message about what
  went wrong. Then it waits for user action.
- Add support for Ubuntu 14.04 and other distributions that use the Upstart init system.
- Make scrollbar thumb draggable.
- Ability to expand cities with multiple servers and configure the app to use a specific server.
- Add firewall rules allowing traffic to the SSDP/WS-discover multicast IP, 239.255.255.250, if
  local area network sharing is activated. This allows discovery of devices using these protocols.

#### macOS
- Add uninstall script that can uninstall and remove all the files installed by the app.

#### Windows
- Extend uninstaller to also remove logs, cache and optionally settings.
- Add installation log (%PROGRAMDATA%\Mullvad VPN\install.log).

### Changed
- The "Buy more credit" button is changed to open a dedicated account login page instead of one
  having a create account form first.
- The CLI command to list relays is now shorter, `mullvad relay list` instead of
  `mullvad relay list locations`.
- Replace WebSockets with Unix domain sockets/Named pipes for IPC. The location
  of the socket can be controlled with `MULLVAD_RPC_SOCKET_PATH`.
- Update the relay list if it's out of date when the daemon starts.
- Move the CLI binary (`mullvad`) on macOS and Linux up one level, so it's installed directly into
  the app installation directory instead of the `resource` directory.

### Fixed
- Fix incorrect window position when using external display.
- Don't auto-connect the daemon on start if no account token is set. This prevents the daemon from
  blocking all internet if logging out from the app.

#### Linux
- The app window is now shown in its previous location, instead of at the center of the screen.
- Remove daemon log, cache and configuration directories during full uninstallation of the app.
- Restart the daemon automatically on upgrade.
- Fix systemd unit file to support older versions of systemd (e.g., in Debian 8).

#### macOS
- Fix edge cases when window's arrow appeared misaligned and pointed to the wrong menubar item.
- Make the pkg installer kill any running GUI process after installation is done. Prevents
  accidentally running an old GUI with a newer daemon.

#### Windows
- Failing to restore DNS settings on daemon start does not make the daemon exit with an error, just
  log the error and continue.


## [2018.2] - 2018-08-13
This release is identical to 2018.2-beta3


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
