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
- Add Kyber1024 KEM algorithm into the Post-Quantum secure key exchange algorithm. This means the
  Quantum-resistant-tunnels feature now mixes both Classic McEliece and Kyber for added protection.
- Add notification dot to tray icon and system notification throttling.
- Add troubleshooting information to some in-app notifications.
- Add setting for quantum resistant tunnels to the desktop GUI.

### Changed
- Update the Post-Quantum secure key exchange gRPC client to use the stabilized
  `PskExchangeV1` endpoint
- Add "auto" setting for the quantum-resistant tunnel feature, and make it the default. If it was
  previously set to off, it will now be set to auto instead. That currently means the same thing as
  "off", but this might change in the future.
- Update OpenVPN to 2.6.0 from 2.5.3.
- Update OpenSSL to 1.1.1t from 1.1.1j.

#### Windows
- Remove automatic fallback to wireguard-go. This is done as a first step before fully
  deprecating it on Windows.

### Fixed
- Fix close to expiry notification not showing unless app is opened once within the last three days
  in the desktop app.
- Fix desktop app not quitting properly after switching from unpinned to pinned window.

#### Android
- Fix adaptive app icon which previously had a displaced nose and some other oddities.


## [2023.1] - 2023-02-20
### Fixed
#### Windows
- Don't fail to show the mullvad-daemon help text if some of the default paths cannot be obtained.


## [2023.1-beta2] - 2023-02-06
### Fixed
- Fix sorting of devices in the "too many devices"-view to properly sort on device creation time.

#### macOS
- Improved reliability of the connectivity check workaround by adding an extra captive portal check
  domain.
- Show "Mullvad VPN" in the Login Items UI instead of "Amagicom AB".
- Detect whether users need to approve the launch daemon in the Login Items UI.


## [2023.1-beta1] - 2023-01-26
### Added
- Add quit button to tray context menu on Linux and Window.
- Add search bar to location list in desktop app.

#### Windows
- Remove all settings when the app is uninstalled silently.

### Changed
- Update Electron from 19.0.13 to 21.1.1.

### Fixed
- When a country is selected, and the constraints only match relays that are not included on the
  country level, select those relays anyway.
- Fix regression where WireGuard relays were connected to over OpenVPN after a couple of failed
  attempts, when the tunnel type was set to `any`.
- Fix missing connect timeout when connecting to a WireGuard relay over TCP.
- Fix failure to apply firewall rules that could occur when connecting timed out.

#### macOS
- Fix fish shell completions when installed via Homebrew on Apple Silicon Macs.

#### Linux
- Remove last filesystem dependency of early boot blocking unit.
- Ensure RPM package removes all application directories when uninstalled.
- Fix architecture field for ARM RPM builds so the app installs on Fedora based distros.

#### Windows
- Ignore adapters that have no valid GUID when removing obsolete Wintun interfaces during install.
  Previously, the installer would abort.
- Revert to using netsh for DNS config, as some Windows builds did not deal with changes correctly.
  `TALPID_DNS_MODULE` can be used to override this.
- Fix deadlock that could occur when the default route changed while initializing split tunneling.

### Removed
#### macOS
- Remove ⌘Q shortcut.

### Security
#### Windows
- DNS loopback traffic is no longer blocked. Note that local resolvers are still unable to forward
  queries to servers that would normally be blocked.


## [android/2022.3] - 2022-11-14
### Added
#### Android
- Add privacy policy link in settings.
- Add initial privacy consent which is showed on each start until approved.


## [android/2022.2] - 2022-10-17
Identical to android/2022.2-beta2 except for updated translations.


## [2022.5] - 2022-10-14
### Fixed
#### Linux
- Don't prevent early boot service from running if logging to a file fails.


## [2022.5-beta2] - 2022-10-05
### Added
- Add custom option to WireGuard port selector.

#### Linux
- Add ARM64 (aarch64) builds. This is the first release with Linux ARM support.

### Changed
- Experimental: Upgrade the support for quantum-resistant WireGuard tunnels to a newer protocol.

### Fixed
- Reset known location when connecting to a custom relay.

#### Linux
- Fix app crashing immediately when using some icon themes.


## [2022.5-beta1] - 2022-09-26
### Added
- Add obfuscation settings under "WireGuard settings".

#### Windows
- The default VPN protocol is slowly being changed from OpenVPN to WireGuard.
  The app fetches the ratio between the protocols from the API.

#### Linux
- GUI: Add electron flags to run Wayland native if in a compositor/desktop known to work well
- Add support for Linux ARM64. No installers are produced yet. But the source code can now
  be built for ARM64.

### Changed
- Reject invalid WireGuard ports in the CLI.
- Reorganize settings into more logical categories.
- Upgrade wireguard-go to 20220703234212 (Windows: v0.5.3).
- Prune bridges far away from the selected relay.
- Stay connected when desktop app is killed or crashes. The only situation where the app now
  disconnects on quit is when the user presses the quit button.
- Update Electron from 18.0.3 to 19.0.13.
- Expand allowed range of multicast destinations to include all of `239.0.0.0/8` (administratively
  scoped addresses), when local network sharing is enabled.
- Default to selecting Sweden as the entry location when using WireGuard multihop. Previously,
  a random location was used.

#### Windows
- Remove dependency on `ipconfig.exe`. Call `DnsFlushResolverCache` to flush the DNS cache.
- Upgrade Wintun to 0.14.1.

#### Linux
- The daemon binary and systemd unit file will now be placed in `/usr/bin/` and
  `/usr/lib/systemd/system` respectively, to aid with starting the system service on systems where
  `/opt` isn't mounted during early boot.

### Fixed
- Connect to TCP endpoints over IPv6 if IPv6 is enabled for WireGuard.
- Fix udp2tcp not working when quantum-resistant tunnels are enabled.
- Quit app gracefully if renderer process is killed or crashes.
- Enable reconnect in blocked state in desktop app.
- Fix error handling during device removal in the desktop app.
- Enable interface settings when app is logged out
- Fix 'mullvad status -v' to include the port of the endpoint when connecting over TCP.
- Check whether the device is valid when reconnecting from the error state.
- Stop reconnecting when the account has run out of time.
- Show out of time view after failing to connect due to being out of time.

#### Windows
- Only use the most recent list of apps to split when resuming from hibernation/sleep if applying
  it was successful.
- Don't fail install if the device tree contains nameless callout driver devices.

### Security
- When the system service is being shut down and the target state is _secured_, maintain the
  blocking firewall rules. Unless it's possible to deduce that the system isn't shutting down and the
  system service is being stopped by the user intentionally. This is to prevent leaks that might
  occur during system shutdown. Fixes 2022 Mullvad app audit issue item `MUL22-02`.

#### Windows
- Upgrade win-split-tunnel driver to version 1.2.2.0. Fixes incomplete validation of input buffers
  that could result in out-of-bounds reads. Fixes 2022 Mullvad app audit issue item `MUL22-01`.

#### Linux
- Added traffic blocking during early boot, before the daemon starts, to prevent leaks in the case
  that the system service starts after a networking daemon has already configured a network
  interface.


## [android/2022.2-beta2] - 2022-09-09
### Changed
#### Android
- Refresh device data when opening the account view to ensure the local data is up-to-date and that
  the device hasn't been revoked.
- Disable settings button during login.

### Fixed
#### Android
- Fix crash sometimes occurring during account creation.
- Fix tunnel info expansion state not remembered during pause and resume.
- Fix crash during some view transitions.
- Fix disabled login button on login failure. Instead, the login button will now still be enabled
  on login failures to let the user re-attempt the login.


## [2022.4] - 2022-08-19
### Added
#### Windows
- Windows daemon now looks up the MTU on the default interface and uses this MTU instead of the
  default 1500. The 1500 is still the fallback if this for some reason fails. This may stop
  fragmentation.

### Fixed
#### Linux
- Fix issue where MTU could not be manually set in the app.
- Lower the max MTU from the automatic MTU detection down to 1380,
  which was the hardcoded default before the automatic detection was implemented.
  This solves issues where the physical interface MTU was set higher than it could
  actually transport.


## [android/2022.2-beta1] - 2022-08-11
### Added
#### Android
- Add device management to the Android app. This simplifies knowing which device is which and adds
  the option to log other devices out when the account already has five devices.

### Changed
#### Android
- Lowered default MTU to 1280 on Android.
- Disable app icon badge for tunnel state notification/status.

### Removed
#### Android
- Remove WireGuard view as it's no longer needed with the new way of managing devices.

### Fixed
#### Android
- Fix unused dependencies loaded in the service/tile DI graph.
- Fix missing IPC message unregistration causing multiple copies of some messages to be received.
- Fix quick settings tile being unresponsive and causing crashes on some devices.
- Fix quick settings tile not working when the device is locked. It will now prompt the user to
  unlock the device before attempting to toggle the tunnel state.
- Fix crash when clicking in-app URL notifications.

### Security
#### Android
- Prevent location request responses from being received outside the tunnel when in the connected
  state.


## [2022.3] - 2022-08-10
This release is for desktop only.

This release is identical to 2022.3-beta3.


## [2022.3-beta3] - 2022-07-28
### Fixed
- Fix showing incompatible relay filtering options in desktop app. The filtering options are now
  dependent on the other filters.

#### Windows
- Fix app occasionally getting stuck in the offline state after being suspended.

#### Linux
- Fixed incompatibility with newer kernel versions (5.19 and up).

### Security
#### Windows
- Fix potential leak window when stopping the service and auto-connect is enabled and always
  require VPN is disabled. When stopped, usually due to a reboot, the daemon would disconnect
  before entering a blocking state.


## [2022.3-beta2] - 2022-06-29
### Fixed
#### Windows
- Fix DNS issue on non-English Windows installations. Don't parse the output of ipconfig.exe
  to determine if the tool succeeded.


## [2022.3-beta1] - 2022-06-27
### Added
- Add option to filter relays by ownership in the desktop apps.
- Experimental: Add support for quantum-resistant PSK exchange to the CLI.

#### Linux
- Automatically attempt to detect and set the correct MTU for Wireguard tunnels.

#### Windows
- Add CLI command for listing excluded processes.

### Removed
#### Linux
- Remove upstart init configuration files

### Fixed
- Display consistent colors regardless of monitor color profile on desktop.
- Fix time added view displayed due to incorrect local clock.

#### Windows
- Be more scrupulous about removing temporary files used by the installer and uninstaller.
- Fix issue where local name resolution fails. This requires users to ensure that non-tunnel
  interfaces are configured correctly to use local custom DNS.
- Configure DNS correctly when the DNS client service is disabled or not responding.


## [2022.2] - 2022-06-13
This release is for desktop only.

This release is identical to 2022.2-beta2.


## [2022.2-beta2] - 2022-06-10
### Added
- Include creation timestamp for devices in the CLI.

### Changed
- List devices on an account sorted by creation date, oldest to newest, instead of alphabetically.

### Fixed
- Fix logout failing if the API cannot be reached in the GUI.


## [2022.2-beta1] - 2022-05-16
### Added
- Extend DNS blocking with the following new categories: "Adult content" and "gambling".
- Obfuscate traffic to the Mullvad API using bridges if it cannot be reached directly.
- Add device management to desktop app. This simplifies knowing which device is which and adds the
  option to log out other devices when there are already 5 connected when logging in.
- Add tray icon tooltip with connection info in desktop app.
- Add relay and bridge constraints for restricting relay selection to rented or Mullvad-owned
  relays. Allows filtering servers by ownership in the CLI.

#### Windows
- Detect mounting and dismounting of volumes, such as VeraCrypt volumes or USB drives,
  and exclude paths from the tunnel correctly when these occur. This sometimes only works
  when the GUI frontend is running.
- Add toggle for split tunneling state.

### Changed
- Update settings format to `v6`.
- Move WireGuard TCP obfuscation settings into `mullvad obfuscation` command in CLI.
- Decrease the size of fonts, some icons and other design elements in the desktop app. This makes it
  possible to fit more into the same area and makes text easier to read.
- Don't block the tunnel state machine while starting the tunnel monitor. This also means that
  the machine will not transition directly from the `disconnected` to the `disconnecting` state
  if an error occurs.
- Change behavior of escape key in the desktop app. It now navigates backwards one step instead of
  to the main view. To navigate back to the main view Shift+Escape can be used.
- Update Electron from 16.0.4 to 18.0.3.
- Randomize bridge selection with a bias in favor of close bridges.
- Make login field keep previous value when submitting an incorrect account number in desktop app.
- Decrease the time it takes to connect to WireGuard relays by sending an ICMP packet immediately.
- Pause API interactions when the daemon has not been used for 3 days.
- Simplified output of `mullvad status` command.

### Fixed
- Fix the sometimes incorrect time added text after adding time to the account.
- Fix scrollbar no longer responsive and usable when covered by other elements.
- Improve tunnel bypass for the API sometimes not working in the connecting state.
- Fix resource leak caused by location check.
- Fix issue where sockets didn't close after disconnecting from WireGuard servers over TCP
  by updating `udp-over-tcp` to 0.2.
- Parse old account history formats correctly when they are empty.
- Use suspend-aware timers for relay list updates and version checks on all platforms.
- Don't attempt to use bridges when using OpenVPN over UDP and bridge mode is set to auto.
- Use the entry endpoint when the relay selector fails to find a relay using the preferred
  constraints and the tunnel protocol is "any". Previously, the entry endpoint was ignored in this
  case.

#### Windows
- Fix "Open Mullvad VPN" tray context menu item not working after toggling unpinned window setting.
- Fix apps not always visible in split tunneling view after browsing for an app and then removing it
  from the excluded applications.
- Fix navigation resetting to main view when toggling the unpinned window setting.
- Log splitting event reason correctly.

#### macOS
- Fix thrashing due to DNS config monitoring.

### Security
#### Windows
- Update split tunnel driver to 1.2.1.0. This fixes potential DNS leaks seen when excluding at least
  one application.


## [android/2022.1] - 2022-03-01
Identical to android/2022.1-beta3 except for a few updated translations.


## [2022.1] - 2022-03-01
This release is for desktop only.

This release is identical to 2022.1-beta2 except that it has translations for new texts in the UI.


## [2022.1-beta2] - 2022-02-22
### Added
- Show warning message when blocking internet while logged out, and make it possible to unblock the
  connection from the login view.

### Fixed
- Prevent settings file being truncated before being read. This caused the daemon to read an empty
  settings file, restore to default settings and log out.
- Improve performance for automatically scrolling text in desktop app.
- Increase availability of the API by allowing to issue requests to the API when connecting to the
  relay even if account data might be invalid.

### Security
- Enable "Always require VPN" by default if the settings cannot be parsed. This reduces the number
  of errors that lead to the daemon unexpectedly starting into non-blocking mode.


## [2022.1-beta1] - 2022-02-14
### Added
- Add information about the always on kill switch in the desktop app.
- Add WireGuard multihop setting and entry location selection to desktop app.
- Add malware blocking to the desktop app. Implemented via DNS on the relays.
- Add changes dialog which will include the most notable changes in each new version.

### Changed
- Keep unspecified constraints unchanged in the CLI when providing specific tunnel constraints
  instead of setting them to default values.
- Obscure account number in account view and add button for copying instead of copying when text is
  pressed.
- Disable logging of translation errors in production. This will among other things prevent error
  messages from translating the country in the disconnected state.
- Update Electron from 15.0.0 to 16.0.4.
- Stop preferring OpenVPN when bridge mode is enabled.
- CLI command for setting a specific server by hostname is no longer case sensitive.
  Example: `mullvad relay set hostname SE9-WIREGUARD` should now work.
- Update the default Shadowsocks password to `mullvad` and cipher to `aes-256-gcm` in the CLI
  when using it to configure a *custom Shadowsocks bridge*. The Mullvad bridges recently changed
  these parameters on port 443 (which is the default port).

#### Windows
- Update wireguard-nt to 0.10.1.
- Make wireguard-nt the default driver for WireGuard. This is used instead of wireguard-go and
  Wintun.
- Increase firewall transaction timeout from 2 to 5 seconds to lower the chance of errors when
  setting the firewall policy.
- Update split tunnel driver to 1.2.0.0. Notably, this driver release allows firewall filters
  added by other software to block excluded apps.

### Removed
#### Windows
- Drop support for pre-Windows 10 systems.

### Fixed
- Always kill `sslocal` if the tunnel monitor fails to start when using bridges.
- Show relay location constraint correctly in the CLI when it is set to `any`.
- Prevent gRPC from trying to run the app-daemon IPC communication through a HTTP proxy when the
  environment variable `http_proxy` is set. This caused the app to fail to connect to the daemon.
- Disable built-in DNS resolver in Electron. Prevents Electron from establishing connections to
  DNS servers set in system network preferences.
- Fix tray context menu showing or executing wrong actions, using wrong language or in other
  ways not update properly.

#### macOS
- Resolve issues with the app blocking internet connectivity after sleep or when connecting to new
  wireless networks.
- Fix issue where the app would get stuck in offline state after a reboot or a reinstall by using
  `route monitor` instead of relying on `SCNetworkReachability` API to infer whether the host is
  offline.

#### Windows
- Fix app size after changing display scale.
- Fix daemon not starting if all excluded app paths reside on non-existent/unmounted volumes.
- Remove tray icon of current running app version when upgrading.
- Allow Mullvad wireguard-nt tunnels to work simultaneously with other wg-nt tunnels.
- Fix notifications on Windows not showing if window is unpinned and hidden.
- Wait for IP interfaces to arrive before trying to configure them when using wireguard-nt.
- Fix panic that occurs in the split tunnel monitor when a path consisting only of a prefix,
  such as "C:", is excluded using the CLI.

#### Linux
- Remove auto-launch file, GUI settings and other files created by the app in user directories, when
  uninstalling/purging.

### Security
- Restrict which applications are allowed to communicate with the API while in a blocking state.
  This prevents malicious scripts on websites from trying to do so. On Windows, only
  `mullvad-problem-report.exe` and `mullvad-daemon.exe` executables are allowed to reach the API,
  whereas on Linux and macOS only root processes are able to reach the API.


## [android/2022.1-beta3] - 2022-02-08
### Fixed
#### Android
- Fix app crash caused by quick settings tile.


## [android/2022.1-beta2] - 2022-01-27
### Fixed
#### Android
- Fix app sometimes crashing during startup on Android TVs.


## [android/2022.1-beta1] - 2022-01-26
### Added
#### Android
- Add toggle for Split tunneling view to be able to show system apps
- Add support of adaptive icons (available only from Android 8).

### Changed
- Gradually increase the WireGuard connectivity check timeout, lowering the timeout for the first
  few attempts.

#### Android
- Improve stability by running the UI and the tunnel management logic in separate processes.
- Remove dialog warning that only custom local DNS servers are supported, since public custom DNS
  servers are now supported.
- Drop support for Android 7/7.1 (Android 8/API level 26 or later is now required).
- Change so that swiping the notification no longer kills the service since that isn't a common way
  of handling the lifecycle in Android. Instead rely on the following mechanisms to kill the
  service:
  * Swiping to remove app from the Recents/Overview screen.
  * Android Background Execution Limits.
  * The System Settings way of killing apps ("Force Stop").
- Change Quick Settings tile label to reflect the action of clicking the tile. Also add a subtitle
  on supported Android versions (Q and above) to reflect the state.
- Hide the tunnel state notification from the lock screen.

### Fixed
#### Android
- Fix banner sometimes incorrectly showing (e.g. "BLOCKING INTERNET").
- Fix tunnel state notification sometimes re-appearing after being dismissed.
- Fix invalid URLs. Rely on browser locale rather than app/system language.
- Automatically disable custom DNS when no servers have been added.
- Fix issue where erasing wireguard MTU value did not clear its setting.
- Fix initial state of Split tunneling excluded apps list. Previously it was not notified the daemon
properly after initialization.
- Fix UI sometimes not updating correctly while no split screen or after having a dialog from
  another app appear on top.
- Fix request to connect from notification or quick-settings tile not connecting if VPN permission
  isn't granted to the app. The app will now show the UI to ask for the permission and correctly
  connect after it is granted.
- Fix quick-settings tile sometimes showing the wrong tunnel state.
- Fix TV-only apps not appearing in the Split Tunneling screen.
- Fix status bar having the wrong color after logging out.


## [2021.6] - 2021-11-17
### Fixed
- Fix the font for Russian. Issue introduced in 2021.6-beta1.
  Bundle a version of Source Sans Pro containing Cyrillic.


## [2021.6-beta1] - 2021-11-03
### Added
#### Windows
- Add black monochromatic tray icon for Windows when using light color for tray.

### Changed
- Update Electron from 11.4.9 to 15.0.0.
- Revamp main view with blurred background behind semi-transparent buttons and switch to
  correct font for logo. Also a slightly less bold font in other parts of the app.

### Removed
- Remove the old Let's encrypt root certificate from the API REST client. Only bundle and
  use the latest certificate.

### Fixed
- Fix desktop app showing a future date for when WireGuard key was generated.
- Fix desktop app split tunneling view to not overflow on very long application names.
- Prevent API requests from being made prior to the tunnel state machine being set up.
  Rarely, failed requests could result in a deadlock.
- Fix segmentation fault when closing app (the GUI).

#### Windows
- Fix detection of Windows 11. Problem reports will now correctly report Windows 11 instead
  of Windows 10.
- Fix race condition in split tunneling initialization. Listen to route changes before reading
  out the default route.
- Fix bug in split tunneling code that could make the kernel driver and mullvad-daemon out of
  sync around which programs should be excluded when the driver took longer to respond.
- Use route-based offline monitoring. Fixes issues where the daemon falsely entered the offline
  state, for example when using virtual switches in Hyper-V.
- Improve repositioning of app window after connecting/disconnecting external monitor.

#### macOS
- Prevent app from showing when dragging tray icon on macOS.
- Move window after dragging tray icon to new position.

#### Linux
- Greatly simplify behavior around custom DNS when using systemd-resolved, by not setting DNS config on interfaces other
  than our tunnel interface.


## [2021.5] - 2021-10-25
This release is for desktop only.

This release is identical to 2021.5-beta2 except that it has translations for new texts in the UI.


## [2021.5-beta2] - 2021-10-19
### Fixed
#### macOS
- Fix issue with getting PF status due to an ABI change on macOS 12 Beta 9.


## [2021.5-beta1] - 2021-10-12
### Added
- Added possibility to filter locations by provider in the desktop app.
- Add ability to use WireGuard over TCP towards all relays via the desktop CLI. However,
  this service is not yet available on all relays. At the time of writing, this only
  works towards `se6-wireguard`, `se9-wireguard` and `se17-wireguard`.
- Add GUI environment variable `MULLVAD_DISABLE_UPDATE_NOTIFICATION`. If set to `1`, GUI
  notification will be disabled when an update is available. This is not intended to be set by
  normal users.
- Add setting for changing between IPv4 and IPv6 for the connection to WireGuard servers on
  desktop.

#### Windows
- Resolve symbolic links and junctions for excluded apps.
- Add opt-in support for NT kernel WireGuard driver. It can be enabled in the CLI.
  Should give better performance. Especially over Wi-Fi.

### Changed
- Only use the account history file to store the last used account.
- Update the out of time-view and new account-view to make it more user friendly.
- Change the app update notification when the suggested version is a beta, to include that it's a
  beta.
- Upgrade OpenVPN from 2.5.1 to 2.5.3.
- Update Electron from 11.2.3 to 11.4.9.
- Move OpenVPN and WireGuard settings in the advanced settings view into separate settings views.
- Return to main view in desktop app after being hidden/closed for two minutes.
- Settings format updated from `v3` to `v5`.

#### Linux
- Always send DNS requests inside the tunnel for excluded processes when using public custom DNS.

#### Windows
- Upgrade Wintun from 0.10.4 to 0.13.
- Reduce tunnel setup time for OpenVPN by disabling DAD.

### Fixed
- Fix link to download page not always using the beta URL when it should.
- Fix deadlock that may occur when the API cannot be reached while entering the connecting state.
- Fix bug causing desktop app to log in if account number field was filled when removing account
  history.
- Fix lack of account expiry updates when using the app in unpinned mode and improve updating of
  account expiry overall.
- Fix incorrect WireGuard relay filtering when exit and entry locations overlap.
- Fix wrong translations when switching to/from unpinned window after changing language in the
  desktop app.
- Fix in-app notification button not working for some notifications.
- Fix incorrectly positioned navigation bar title when navigating back to a scrolled down view.
- Fix connectivity check for WireGuard multihop when the exit hop is down.
- Fix incorrect location and connection status while disconnecting and incorrect location in the
  beginning while connecting in the desktop app.
- Improve responsiveness of the controls and status text in the main view in the desktop app.
- Read macOS scrollbar visibility settings to decide wheter or not the scrollbars should hide when
  not scrolling.
- Fix IPv6 connections to WireGuard servers by not dropping select neighbor advertisements and
  solicitations.

#### Linux
- Make offline monitor aware of routing table changes.
- Assign local DNS servers to more appropriate interfaces when using systemd-resolved.
- Disable DNS over TLS for tunnel's DNS config when using systemd-resolved.
- Fix DNS when combining a static resolv.conf with ad blocking DNS.
- Check connectivity correctly on IPv6-only networks.

#### Windows
- Fix failure to restart the daemon when resuming from "fast startup" hibernation.
- Fix OpenVPN not responding to shutdown signals when they are sent early on, causing it to close
  after 30 seconds.
- Disable notification actions for persistent notifications since they were called when pressing
  close.
- Remove deleted network devices from consideration in the offline monitor. Previously, the offline
  monitor may have falsely reported the machine to be online due to a race condition.
- Recover firewall state correctly when restarting the service after a crash. This would fail when
  paths were excluded.
- Fix daemon not starting when a path is excluded on a drive that has since been removed.
- Prefer WireGuard if the constraints preclude OpenVPN and the tunnel protocol is "auto", instead
  of failing due to "no matching relays".
- Retry tunnel device creation multiple times to work around issues early after boot or hibernation.


## [2021.4] - 2021-06-30
This release is for desktop only.

This release is identical to 2021.4-beta1 except that it has translations for new texts in the UI.


## [2021.4-beta1] - 2021-06-09
This release is for desktop only.

### Added
- When `MULLVAD_MANAGEMENT_SOCKET_GROUP` is set, only allow the specified group to access the
  management interface UDS socket. This means that only users in that group can use the CLI and GUI.
- Support WireGuard over TCP for custom VPN relays in the CLI. (Our relays don't support this yet).
- Make app native on Apple Silicon.
- Support WireGuard multihop using an entry endpoint constraint in the CLI.
- Add Ad and tracker blocking to the desktop app. Implemented via DNS on the relays.

#### Windows
- Add split tunneling as a beta feature. Allows excluding some applications from the VPN tunnel.

### Changed
- Upgrade OpenVPN from 2.5.0 to 2.5.1.
- Replace CLI command `mullvad custom-dns` with the new command `mullvad dns`.
- Upgrade wireguard-go to `20210521230051` (Windows: v0.3.14)

#### Linux
- Only allow packets with the mark set to `0x6d6f6c65` to communicate with the relay server.
  Previously, bridges were expected to run as root instead.
- Use an ICMP socket instead of relying on a `ping` binary in `$PATH` to establish if a tunnel is
  working.

#### macOS
- Update shape of macOS icon to be in line with Apple's guidelines.

### Fixed
- Fix relay selection failing to pick a WireGuard relay when no tunnel protocol is specified.
- Fix time left not always being translated in desktop app settings.
- Fix API address cache to use the supplied ports instead of always using port 443.
- Do not try to parse an empty account history.

#### Windows
- Prevent tray icons from being extracted to `%TEMP%` directory.
- Fix failure to create Wintun adapter due to a residual network interface by upgrading Wintun to
  0.10.4.
- Wait indefinitely for IP interfaces to attach to the tunnel device to prevent early timeouts,
  and errors setting interface metrics.
- Prevent Microsoft Store from dropping packets in WireGuard tunnels.

#### Linux
- Fix find `mullvad-vpn.desktop` in `XDG_DATA_DIRS` instead of using hardcoded path.

#### MacOS
- Set correct permissions for daemon's launch file in installer.
- Fix downgrades on macOS silently keeping previous version.
- Fix other menubar context menus not always closing when opening app on macOS 11.

### Security
#### Linux
- Drop packets being *forwarded* unless they are approved by the same rules as incoming or outgoing
  traffic.


## [2021.3] - 2021-04-28
This release is for desktop only.

This release is identical to 2021.3-beta2


## [2021.3-beta2] - 2021-04-22
This release is for Windows only.
This release has no functional changes compared to 2021.3-beta1. The only
difference is that the installer and the binaries are signed with our
new code signing certificate. The old one is expiring.

The new certificate is brand new and Windows does not really trust it yet.
So there might be warnings when installing this for a while.


## [2021.3-beta1] - 2021-04-13
This release is for desktop only.

### Added
- Preserve log of old daemon instance when upgrading on Desktop.

#### Linux
- Always enable `src_valid_mark` config option when connecting to allow policty based routing.

### Changed
- Allow whitespace in account token in CLI.
- Read account token from standard input unless given as an argument in CLI.
- Make WireGuard automatic key rotation interval mandatory and between 1 and 7 days.
- Show default, minimum, and maximum key rotation intervals in CLI.
- Attempt to send problem reports using other endpoints if using the primary one fails.
- Upgrade wireguard-go to version 20210225140808 (Windows: v0.3.8)
- Settings format updated to `v3`.

### Fixed
- Fix GUI not showing correct view if disconnected from the daemon during app startup.
- Fix incorrectly displayed "inconsistent version" text in settings if disconnected from daemon on
  startup.

#### Linux
- Further improve offline monitor to properly receive `ENETUNREACH`.

### Security
- Always reconnect appropriately after an upgrade. Previously, installing the app twice in
  succession, with auto-connect disabled, would cause it to re-launch in the disconnected state.


## [android/2021.1] - 2021-05-04
This release is for Android only.

This release is identical to android/2021.1-beta1.
This is our first non beta release for the Android platform!


## [android/2021.1-beta1] - 2021-04-06
This release is for Android only. From now on, Android releases will have this new header format
that is the same as the git tag they receive: `android/<version>`.

### Added
- Enable isolation of the Electron renderer process to protect against potentially malicious third
  party dependencies.
- Add 51820 to list of WireGuard ports in app settings.
- Add option to connect to WireGuard relays over IPv6.
- Add Burmese translations.

#### Android
- Allow reaching the API server when connecting, disconnecting or in a blocked state.
- Add FAQs & Guides menu entry to the Settings screen.
- Add TV banner for better user experience and requirements.
- Style StatucBar and NavigationBar to make our app a bit more beautiful.

### Changed
- Update Electron from 11.0.2 to 11.2.1 which includes a newer Chromium version and
  security patches.
- Allow provider constraint to specify multiple hosting providers.
- Only download a new relay list if it has been modified.
- Connect to the API only via TLS 1.3
- Shrink account history capactity from 3 account entries to 1.

#### Android
- WireGuard key is now rotated sooner: every four days instead of seven.

#### Windows
- Upgrade Wintun from 0.9.2 to 0.10.1.

### Fixed
- Fix delay in showing/hiding update notification when toggling beta program.
- Improve responsiveness when reconnecting after some failed connection attempts.

#### Windows
- Fix "cannot find the file" error while creating a Wintun adapter by upgrading Wintun.
- Retry when creating a WireGuard tunnel fails due to no default routes being found.

#### Linux
- Stop using NM for managing DNS if it's newer than 1.26.
- Fix DNS issues where NM would overwrite Mullvad tunnel's DNS config in systemd-resolved.
- Fix issues with hosts where the firewall is doing reverse path filtering.

#### Android
- Fix input area sometimes disappearing when returning to the Login screen.


## [2021.2] - 2021-02-18
This release is for desktop only.

This release is identical to 2021.2-beta1

## [2021.2-beta1] - 2021-02-17
This release is for desktop only.

### Fixed
#### Linux
- Fix crash when trying to apply IPv6 routes for OpenVPN when IPv6 is disabled.
- Ignore failure to add IPv6 split-tunneling routing rules when they fail due to IPv6 being
  unavailable.

#### Windows
- Fix failure when Wintun adapter name conflicts with that of a non-Wintun adapter.
- Fix detection of SetupAPI patch when not installed by KB2921916.


## [2021.1] - 2021-02-10
This release is for desktop only.

### Fixed
- Fix crash on desktop when entering problem report view when using an outdated app version.


## [2021.1-beta2] - 2021-02-03
This release is for desktop only.

### Fixed
- Make it impossible to apply an empty list of custom DNS server IPs.
  Stops the daemon from getting stuck in the offline state on macOS.

#### Windows
- Only load and unload the Wintun driver once instead of on every connection.
  Will hopefully fix an issue where it gave an error


## [2021.1-beta1] - 2021-01-25
This release is for desktop only.

### Added
- Add header containing OS version to version-check API call to enable OS specific compatibility and
  vulnerability checks.
- Add `TALPID_DISABLE_OFFLINE_MONITOR` environment variable to allow users to disable offline
  detection.

#### Android
- Allow to configure the tunnel to use custom DNS servers.
- Show only applications that has INTERNET permission on split tunnel screen.

#### Linux
- Improved compatiblitiy with newer versions of systemd-resolved.
- Add version data to problem reports for the following software: the kernel, NetworkManager,
  WireGuard kernel module, SystemD.

### Changed
- Allow the API to be accessed while in a blocking state.
- Prefer the last used API endpoint when the service starts back up, as well as in other tools such
  as the problem report tool.
- Migrate cache to a directory readable by all users, consistent with Android and Linux.
- Change the default value of the GUI auto-connect setting to "off" and respect the setting when
  logging in.

#### Linux
- Improve offline check to query the routing table to allow users to use a bridged adapter as their
  primary interface.

### Fixed
- Fix Turkish translations for on/off in the bridge settings. They were inverted, so it was
  confusing to change the setting.
- Stop returning bogus version information when there is no version cache.

#### Linux
- Fix missing app window icon in Xfce.

#### Windows
- Resolve single-label hostnames correctly.

### Security
- Default to connecting when the daemon starts if the target state cache cannot be read or parsed.

#### Linux
- Prevent the private tunnel IPv6 address from being detectable on a local network when using
  OpenVPN by correctly applying the fix for
  [CVE-2019-14899](https://seclists.org/oss-sec/2019/q4/122).

#### Windows
- Deny network access to the OpenVPN plugin pipe, which allowed for anonymous write access when
  passwordless file sharing was enabled.


## [2020.8-beta2] - 2020-12-11
This release is for desktop only.

### Added
#### Android
- Add a screen to view the logs included in a problem report.

### Changed
- Upgrade wireguard-go to v0.0.20201118.
- Reduce logging about time outs when conneting to a WireGuard tunnel.

### Fixed
#### Android
- Fix login appearing to be cancelled after leaving the login screen while logging in.
- Fix login input area missing some times when opening the login screen.


## [2020.8-beta1] - 2020-11-30
This release is for Android only.

### Added
- Improve accessibility in the desktop app. UI elements now have the correct role
  and are labeled to allow usage of a screen reader.
- Add `--wait` flag to `connect`, `disconnect` and `reconnect` CLI subcommands to make the CLI wait
  for the target state to be reached before exiting.
- Navigate back to the main view when escape is pressed.
- Add support for custom DNS resolvers on Windows, macOS and Linux. For now only resolvers in the
  local network will work properly.

#### Windows
- Add setting that unpins the window from the tray icon to let the user move it around freely.

### Changed
- Never use DNS to get the IP to contact the Mullvad API over. Instead a list of IPs is bundled
  with the app, and updates are fetched from the API itself. This list is then shuffled and used
  to pick a way to reach the API. This avoids censored/fake DNS responses and increases the
  likelihood to be able to talk to the API.
- Remove WireGuard keys during uninstallation after the firewall is unlocked.
- Rename CLI subcommand `mullvad relay set relay` to `mullvad relay set hostname`.
- Upgrade OpenVPN from 2.4.9 to 2.5.0.
- Upgrade Electron from 8.5.2 to Electron 11.0.2.

#### Android
- Remove the Quit button.
- Add button to remove account and WireGuard key from history in the login screen.
- Improve navigation in the app using a keyboard, so that touchless devices (like TVs) can be used
  more smoothly.
- Run app in landscape mode on TVs.
- Try to connect even if VPN permission is denied, so that the app shows an error message saying
  that the VPN permission was denied.

#### Windows
- Fully uninstall the old app when performing a downgrade. This solves the problem of downgrades
  not being able to migrate from something newer. For example it fully removes any blocking
  firewall rules a newer app might have put in place.
- Use Wintun instead of the OpenVPN TAP driver for OpenVPN.

#### Linux
- Increase NetworkManager device readiness timeout to 15 seconds.
- Set up routes for OpenVPN using our route manager instead of relying on OpenVPN to do it.
- Use rule-based routing and static routes. Avoids monitoring and duplicating the main routing
  table into a separate table.

### Fixed
- Fix missing map animation after selecting a new location in the desktop app.
- Fix crash on older kernels which report a default route through the loopback interface.

#### Android
- Fix connect action button sometimes showing itself as "Cancel" instead of "Secure my connection"
  for a few seconds.
- Fix the notification sometimes leaving the foreground and becoming dismissable even if the UI was
  still visible.
- Fix crash if connection to service is lost while opening the Split Tunneling settings screen.
- Fix rare crash that could occur when the tunnel state changes when showing or hiding the quick
  settings tile.
- Fix app starting by itself sometimes.
- Fix apps not being excluded from the tunnel sometimes if auto-connect was enabled.
- Fix crash that happened sometimes when closing the app or when requesting from the notification
  or the quick-settings tile for the app to connect or disconnect.
- Fix app showing that it was blocking connections when it wasn't when VPN permission was denied.
- Fix internet not working for a minute or two after changing Allow LAN setting.

#### Windows
- Fix log output encoding for Windows modules.
- Fix app not appearing on top in some situations when pressing the tray icon.
- Fix memory leak in Windows firewall code.

#### Linux
- Stop reconnecting when using WireGuard and NetworkManager.
- Reset DNS config correctly when the tunnel monitor unexpectedly goes down.
- Set search domains in NetworkManager's DNS configuration, resolving issues where NetworkManager
  is used to manage DNS via systemd-resolved.
- Fix incorrect version string in .deb installer causing downgrade warnings when upgrading from beta
  to stable.
- Fix memory leak in firewall code via updating `nftnl` dependency.
- Handle IPv6 traffic correctly using `mullvad-exclude` when there is no default route to any
  non-tunnel interface.
- Fix issues managing DNS when dnsmasq is used with NetworkManager.
- Fix issues with managing kernel WireGuard device via NetworkManager.
- Disable NetworkManager's connectivity check before applying firewall rules to avoid triggering
  NetworkManager's [bug](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/312#note_453724)

### Security
- Restore the last target state if the daemon crashes. Previously, if auto-connect and
  "Always require VPN" were disabled, the service would reset the firewall upon starting back up,
  even if the tunnel was up when the crash occurred.
- Add firewall rules for `mullvad-exclude`, i.e. split tunneling, that disallow all traffic in the
  tunnel other than non-custom DNS traffic. This prevents leaks into the tunnel.
- Force OpenVPN to use TLS 1.3 or newer.

#### Windows
- Block all traffic received or sent before the BFE service and daemon service have started during
  boot, if "Always require VPN" or auto-connect is enabled.


## [2020.7] - 2020-11-18
This release is identical to 2020.7-beta2.
This release has no differences for Windows and macOS compared to 2020.6


## [2020.7-beta2] - 2020-11-16
This release is for desktop only. It only has changes for Linux

### Fixed
#### Linux
- Parse routes more permissively and log parsing errors less verbosely.


## [2020.7-beta1] - 2020-11-10
This release is for desktop only. It only has changes for Linux

### Fixed
#### Linux
- Order routes by prefix size in ascending order when applying them.
  Fixes an issue where seemingly manually added routes would be returned
  from the kernel in an order which can't be applied.
- If possible, use NetworkManager to create a WireGuard interface so that DNS can be managed
  via NetworkManager as well. This fixes the issue where the daemon will reconnect
  spuriously when using NetworkManager and WireGuard.
- Fix route parsing bug in route monitor by ignoring loopback routes.
- Apply DNS config quicker when managing DNS via NetworkManager.
- When NetworkManager is managing /etc/resolv.conf but ultimately using systemd-resolved, use
  systemd-resolved directly to manage DNS.
- Only use WireGuard kernel implementation if DNS isn't managed via NetworkManager.


## [2020.6] - 2020-10-20
This release is for desktop only.

This release is identical to 2020.6-beta3 except updated GUI translations


## [2020.6-beta3] - 2020-10-06
This release is for desktop only.

### Added
#### Linux
- Add support for WireGuard's kernel module if it's loaded.
- Add tray context menu with actions.

### Changed
#### Linux
- Open and focus app when opened from context menu instead of toggling the window.

### Fixed
- Start key rotation when WireGuard key is first created.
- Remove firewall filters (unblock internet access) when "Always require VPN" is enabled and the app
  is uninstalled.

#### Android
- Fix rare crash that could happen when starting the background service.
- Fix rare crash that happened with large text sizes and long location names on the main screen.
- Fix UI not updating in split screen mode when the window is unfocused.
- Fix split tunneling not being correctly configured after restarting the app.
- Fix app reopening after pressing the Quit button because app was running multiple tasks.
- Fix inconsistent behavior of the quick-settings tile when logged out. It would sometimes enter the
  blocking state and sometimes open the UI for the user to login. Now it always opens the UI.
- Mark the VPN connection as not metered, so that Android properly reports if the connection is or
  isn't metered based solely on the underlying network, and not on the VPN connection.

#### Linux
- Fix split tunneling rules preventing `systemd-resolved` from performing DNS lookups for excluded
  processes.
- Honor routes other than the default route with `mullvad-exclude`. This is mainly to improve
  routing within LANs.

### Security
- Stop resetting the firewall after an upgrade to not leak after an upgrade.


## [2020.6-beta2] - 2020-08-27
This release is for Android only.

### Added
- Add CLI command to set the location constraint via `mullvad relay set relay HOSTNAME`.
- Add a provider relay constraint, which restricts relay selection to a given hosting provider.
- Include hosting providers in the CLI for `mullvad relay list` and `mullvad bridge list`.

### Changed
- Use gRPC for communication between frontends and the backend instead of JSON-RPC.
- Show a warning in the CLI if the provided location constraints don't match any known relay.

### Fixed
- Fix high CPU usage in 2020.6-beta1. This was due to an incorrectly initialized stream in the
  relay list updater.
- Fix the relay list not being updated in 2020.6-beta1 after the daemon has started.

#### Android
- Fix possible crash when starting the app, caused by trying to use JNI functions before the library
  is loaded.
- Fix crash when selecting the whole text entered for the voucher code and then deleting it in the
  Redeem Voucher dialog.
- Show "Exclude applications" header if needed when entering the "Split tunneling" screen.
- Fix check for update versions and check for support for current version.
- Fix crash that could happen when leaving the Select Location screen.
- Don't show out-of-time notification for newly created accounts.


## [2020.6-beta1] - 2020-08-20
This release is for Android only.

### Added
- Show system notification when account has expired.
- Add fish shell completions for the mullvad CLI.
- Reconnect with a new key when WireGuard key is rotated automatically, previously the tunnel would
  time out before reconnecting.

#### Linux
- Add split tunneling menu under advanced settings in Linux app.

#### Android
- Add split-tunnelling, allowing apps to be configured to be excluded from the tunnel.
- Add localized app messages.

### Changed
- Upgrade from Electron 7 to Electron 8.
- Change version string parsing to never suggest the user to upgrade to an older version.
- Make connectivity checker more resilient to suspension.
- Make uninstaller on desktop platforms attempt to remove WireGuard keys from accounts.
- Make important notifications not timeout on macOS and remain in the notification list on Linux.
- Add exponential backoff to relay list downloader.
- Display the original block reason in the non-blocking error state, and why applying the blocking
  policy failed.
- Don't show account time expired notification for newly created accounts.

#### Android
- Show a system notification when the account time will soon run out.
- Changed how the Select Location screen scrolls so that more items can be viewed at the same time.

#### Windows
- Upgrade Wintun from 0.7 to 0.8.1.
- Display causes of firewall errors in the GUI.

#### Linux
- Allow users to specify `net_cls` controller mountpoint via `TALPID_NET_CLS_MOUNT_DIR`. The
  specified mountpoint will only be used if the controller isn't mounted already.

### Fixed
- Fix connectivity monitor for WireGuard not disconnecting from a relay when connectivity is lost.
- Forward firewall errors to the GUI in the connecting state, instead of showing a generic message
  about failing to start the tunnel.

#### Windows
- Fix window flickering by disabling window animations.
- Fix WireGuard not connecting if IPv6 is disabled in the adapter or OS. `libwg` would time out
  waiting for an IPv6 interface to become available.

#### Android
- Fix Connect screen sometimes becoming unusually tall. This ended up causing the screen to be
  scrolled up and made the UI elements unable to be seen until the user scrolled down.
- Fix connect action from quick-settings tile or notification sometimes opening the UI instead of
  connecting.
- Fix notification sometimes not being dismissible.
- Fix toggle switch sometimes getting stuck.

#### Linux
- Fix `systemd-resolved` DNS management by not parsing `/etc/resolv.conf`.
- Fix issue where DNS configuration would not be reset when NetworkManager was used and the daemon
  was stopped ungracefully. This persisted after reboots.


## [2020.5] - 2020-06-25
### Added
- Add Korean, Polish and Thai languages to the desktop app.


## [2020.5-beta2] - 2020-06-16
### Added
#### Android
- Add buttons to buy credit and redeem voucher in Account screen.
- Show a notification banner warning when the account time will soon run out.

### Changed
- Send an ICMP reject message or TCP reset packet when blocking outgoing packets to prevent
  timeouts.

#### macOS
- Use `SCNetworkReachability` to help determine connectivity of host. Helps bring the app online
  faster when the computer wakes up from sleep.

#### Android
- Show the remaining account time in the Settings screen in days if it's less than 3 months.
- Prevent commands to connect or disconnect to be sent when the device is locked.
- Make all screens scrollable to better handle small screens and split-screen mode.

### Fixed
- Show both WireGuard and OpenVPN servers in location list when protocol is set to automatic on
  Linux and macOS.
- Fix missing in app notification about unsupported version.
- Prevent auto-connect on login if the account is out of time.
- Fix race that caused WireGuard key upload to fail which could cause the "too many keys" error and
  the tunnel to invalidly fall back to OpenVPN.

#### Android
- Fix crash when that happened sometimes when the app tried to start the daemon service on recent
  Android versions.
- Fix quitting the app sometimes failing.
- Fix WireGuard key status events being lost by the UI, causing stale information to be shown.
- Fix time left in account not showing in settings screen.
- Fix attempt to connect when the app doesn't have the VPN permission.
- Fix crash that happened sometimes when the WireGuard key was loaded too quickly.
- Fix crash when entering split-screen mode whilst on the Report a Problem screen.
- Fix invalid back stack history when connection to service is lost and the app returns to the
  launch screen.
- Fix app leaving settings screen when entering split-screen mode.
- Fix app sometimes leaving Welcome screen prematurely after creating an account.

#### Windows
- Fix race in network adapter monitor that could result in data corruption and crashes.
- Upgrade `miow` dependency to stop daemon from crashing when the management interface named pipes
  were accessed with `accesschk.exe` and some web browsers.
- Fix race that may rarely occur during install when obtaining the GUID of a newly created TAP
  adapter.

### Security
- Tighten the firewall rules that were allowing traffic to the relay server over the physical
  network interface. On Linux and macOS now only processes running under root are allowed to send
  traffic to this port and IP. On Windows only the Mullvad VPN binaries are allowed to send.
  This fixes audit ticket [`MUL-02-002`].

#### Windows
- Tighten the firewall rule allowing traffic on port 53 to the relay server IP on the physical
  interfaces if the VPN tunnel is established on port 53 to only allow UDP. This fixes
  audit ticket [`MUL-02-004`].
- Deny access to the management interface named pipe for the `NT AUTHORITY\NETWORK` group.
  This makes the named pipe no longer accessible under the `IPC$` network share.
  This fixes audit ticket [`MUL-02-007`].

#### Android
- Ignore touch events when another view is shown on top of the app in order to prevent tapjacking
  attacks. Fixes audit ticket [`MUL-02-003`].
- Prevent screens showing potentially sensitive data from being recorded. Fixes audit
  ticket [`MUL-02-003`].

[`MUL-02-002`]: audits/2020-06-12-cure53.md#identified-vulnerabilities
[`MUL-02-003`]: audits/2020-06-12-cure53.md#miscellaneous-issues
[`MUL-02-004`]: audits/2020-06-12-cure53.md#miscellaneous-issues
[`MUL-02-007`]: audits/2020-06-12-cure53.md#identified-vulnerabilities

## [2020.5-beta1] - 2020-05-18
### Added
- Add a new Let's Encrypt root certificate.

#### Android
- Add possibility to create account from the login screen.
- Add welcome screen for newly created accounts.
- Allow submitting voucher codes to add time to the account.
- Add Out Of Time screen for user to add more time to account once it expires.

### Changed
- Move location of the account data (including the WireGuard keys), so that it isn't lost when the
  system cache is cleaned.
- Rename "Block when disconnected" setting to "Always require VPN" and add additional explanation
  of the setting.
- Embed TLS certificates used for HTTPS into the binary rather than loading them from disk at
  runtime.
- Ignore case when setting the relay or bridge location in the CLI.
- Upgrade OpenVPN from 2.4.8 to 2.4.9 and the OpenSSL version it uses from 1.1.1d to 1.1.1g.
- Upgrade shadowsocks-rust to version 1.8.10.
- Always enable the beta program when running a beta version.
- Increase relay list download failure retry interval from 5 to 15 minutes. And from 5 seconds
  to 15 minutes for the WireGuard key rotation retry interval.

#### Android
- Adjust the minimum supported Android version to correctly reflect the supported versions decided
  in 2020.4-beta2. The app will now only install on Android 7 and later (API level 24).

### Fixed
#### Android
- Fix crash when leaving WireGuard Key screen while key is still verifying.
- Fix crash that sometimes happens right after some other unrelated crash.
- Fix app not connecting when pressing the notification or quick-settings tile when the service
  isn't running. It would previously just open the app UI and stay in the disconnected state.
- Fix crash when requesting to connect from notification or quick-settings tile.
- Fix version update notifications not appearing.
- Fix UI losing any settings updates that happen after leaving the app and then coming back.
- Fix account expiration date disappearing in some circumstances.
- Fix notification reappearing after quitting the application.
- Retry when fetching account expiration fails.


## [2020.4] - 2020-05-12
This release is identical to 2020.4-beta4


## [2020.4-beta4] - 2020-05-06
### Fixed
- Fix bogus or absent update notifications on the desktop app due to incorrect deserialization of a
  struct sent from the daemon.

#### Android
- App will now use packaged relay list if it's newer than the cached one.
- Fix relay list sort order.

#### Windows
- Remove all log files on uninstall. Clear install.log on upgrades.


## [2020.4-beta3] - 2020-04-29
### Added
- Add shell completions for the mullvad CLI. Installed for bash and zsh on Linux and zsh on macOS.

### Changed
- Downgrade to Electron 7 due to issues with tray icon in Electron 8.
- Use rustls instead of OpenSSL for TLS encryption to the API and GeoIP location service.

#### Windows
- When required, attempt to enable IPv6 for network adapters instead of failing.

#### Android
- Update the WireGuard Key screen so that it looks the same as on the desktop app. It is now reached
  through the Advanced settings screen.

### Fixed
- Enable IPv6 in WireGuard regardless of the specified MTU value, previously IPv6 was disabled if
  the MTU was below 1380.

#### Windows
- Improve offline detection logic.
- Enable missing IPv6 interface on the WireGuard TUN adapter when it has been disabled.

#### Android
- Change button colors on problem report no email confirmation dialog to match the desktop version.
- Fix crash when attempting to run app from the non-default location, such as the SD card or from a
  different user profile.

### Security
#### macOS
- Ship native Node modules unpacked to prevent malware checks by macOS on each run. The malware
  checks delayed app startup when "block when disconnected" was enabled and performed system network
  requests to Apple.

#### Android
- Fix failure to create tunnel when app is started with auto-connect enabled. This would sometimes
  lead to a traffic leak.


## [2020.4-beta2] - 2020-04-08
### Added
- Add possibility to create account in the desktop app.
- Add possibility to pay with voucher in the desktop app.

#### Android
- Add WireGuard MTU setting.

### Changed
- Allow `fc00::/7` instead of `fd00::/8` in the firewall when local network sharing is enabled.
  Should unblock all unique local addresses.
- Upgrade from Electron 7 to Electron 8.
- Formalize what operating system versions we support in the [readme](README.md). In practice
  this means dropped support for Android 5 and 6 and Fedora 28 and 29 right away, Ubuntu 16.04
  support will end as soon as Ubuntu 20.04 comes out.

#### Windows
- Windows 7 only: Address packet loss issues with OpenVPN on some systems by reverting the TAP
  adapter driver to an older NDIS 5 driver.


## [2020.4-beta1] - 2020-03-30
### Added
- Add signal handlers on Linux, macOS and Android to better log critical faults with the daemon.
- Add WireGuard MTU setting to desktop app.
- Add option to receive notifications about new beta releases.

#### Android
- Add option to enable auto-connecting behavior
- Include an initial relay list in the APK so that the app can connect to the VPN even if it fails
  to connect to the API after it is installed.
- Add a reconnect button to disconnect and connect again without closing the tunnel device to avoid
  leaking any data during the reconnection.
- Add quick settings tile to control the tunnel state.
- Enable IPv6 traffic through the tunnel.

### Changed
- Prefer WireGuard when tunnel protocol is set to _auto_ on Linux and MacOS.
- Wait for tunnel state machine to properly shut down, cleaning up the firewall properly on Windows
  during the daemon shutdown.
- Switch to new logo.
- Show better message when the app failed to block all connections after an error.

### Fixed
- Fix bug that could lead to Javascript error dialog to appear upon the desktop app termination.
- Fix rendering glitch in the map and improve the map's resource usage.

#### macOS
- Fix firewall rules to properly handle DNS requests over TCP when "Local network sharing" is
  disabled. Previously DNS requests over TCP would timeout.

#### Android
- Fix notification action button not working when requesting to connect the tunnel after being
  disconnected for a long time.
- Make the settings screen scrollable, so that the quit button is reachable on small screens.
- Fix connectivity listener leak causing possible battery usage increase.
- Fix crash that could sometimes happen when restarting the background service.
- Fix incorrect location information sometimes shown in main screen.

#### Windows
- Fix bug where failing to initialize the route manager could cause the daemon to get stuck in a
  blocked state. This only affected WireGuard.

### Security
- When upgrading or reinstalling while connected, exit the daemon in a blocking state to prevent
  unintended leaks. This only affects upgrades from this release.

#### Windows
- Fix issue in daemon where the `block_when_disconnected` setting was sometimes not honored when
  stopping the daemon. I.e. traffic could flow freely after the daemon was stopped.

#### Android
- Fix issue where IPv6 traffic could leak outside of the tunnel.


## [2020.3] - 2020-02-20
This release is identical to 2020.3-beta1


## [2020.3-beta1] - 2020-02-20
### Security
- Fix stack overflow caused by WireGuard key rotation timers. When the daemon crashed it was
  restarted automatically. But it did not connect (depending on settings), leaving a leak.


## [2020.2] - 2020-02-13
This release is identical to 2020.2-beta1


## [2020.2-beta1] - 2020-02-13
### Added
- Add reconnect button to the desktop app.
- Add monochrome option for the tray icon on Windows and Linux.
- Show OS notification when account is close to expiry on desktop platforms.
- Warn users running old app versions when creating problem report.

#### Android
- Add option to enable or disable local network sharing.
- Show account history in login fragment

### Changed
- Change project copyright and company name from Amagicom AB to Mullvad VPN AB
- Only reconnect when settings change if a relevant tunnel protocol is used.
- Adjust padding of tray icon on Windows and Linux to better match other icons.
- Change the zoomlevel of the map in the desktop app to make it less zoomed in.
- Bundle new API IP with the app (Old: 193.138.218.73, new: 193.138.218.78)

### Removed
- Remove city/country labels on map in the desktop app.

### Fixed
- Fix app sometimes getting stuck in connecting state when using WireGuard.

#### Android
- Fix crash when removing the service from foreground on Android versions below API level 24.
- Fix crash that happened in certain situations when retrieving the relay list.
- Fix crash caused by initialization race condition.

#### Windows
- Fix "exhausted namespace" installation error on some non-English systems.

### Security
- Stop DNS leak that could happen on all desktop platforms if "Local network sharing" was enabled
  and the device had a default DNS resolver on the local private network. The leak could happen
  during these states: While connecting, when blocked due to an error happening and when
  disconnected if the "block when disconnected" setting was enabled.
  This issue has been present on all previous versions of the app.

#### Windows
- Prevent DNS leak that could happen while connected if "Local network sharing" was enabled
  and the device had a default DNS resolver on the local private network. This issue was
  only present in the 2020.1 release.


## [2020.1] - 2020-02-10
This release is identical to 2020.1-beta1


## [2020.1-beta1] - 2020-02-05
### Added
- Add translations for Finnish and Danish.
- Copy WireGuard key when clicking on it.

#### Windows
- Sign all binaries in the app instead of just the installer.

### Changed
- Increase OpenVPN ping timeout from 20 to 25 seconds. Might make working tunnels disconnect
  a bit less frequently.
- Use traffic data from WireGuard to infer connectivity, instead of continuously pinging.
  Should improve stability of the connection and reduce power use.
- Update `wireguard-go` to `v0.0.20200121`
- Remove WireGuard keys from accounts when they are removed from the local account history.
- Upgrade from Electron 6 to Electron 7.
- Disable WireGuard protocol option if there's no WireGuard key.

#### Android
- Wait for traffic to be routed through the tunnel device before advertising blocked state.
- Connect automatically if `MullvadVpnService` is started with an intent which
  has the `android.net.VpnService` action. Effectively, this should enable
  _Always On_ behavior on Android versions where it's supported.
- Allow notification to be dismissed when the UI is not shown and the tunnel is disconnected.

#### Windows
- Use a branded TAP driver for OpenVPN to prevent conflicts with other software and solve issues
  related to driver upgrades. Also use the NDIS 6 driver on Windows 7.
- Be more aggressive when installing routes, in effect taking ownership of existing duplicate route
  entries. This allows the daemon to initialize properly even if a previous instance did not have a
  clean shutdown.

### Fixed
- Don't try to replace WireGuard key if account has too many keys already.
- Fix bogus update notification caused by an outdated cache.
- Fix layout issues when showing messages in WireGuard key view.
- Fix translation of "System default" after selecting "System default" in language settings.

#### Windows
- Fix regression due to which a TAP adapter issue was not given as the specific block reason when
  the tunnel could not be started.
- Fix occasional failure to shut down the old daemon process during installation by killing it if
  necessary.
- Make WireGuard work with IPv6 enabled even if there is no functioning TAP adapter for OpenVPN.
- Restart daemon when coming back from system hibernation with terminated user session, since
  it's perceived as a cold boot from the user's perspective, so the app should act accordingly.
- Change the optimization level for releases from the default value to `s`, as a temporary fix for
  the system service crashing on Windows for newer CPU models.

#### Android
- Fix notification message to not show `null` version when version check cache is stale right
  after an update.
- Fix `null` pointer exception when connectivity event intent has no network info.
- Fix fast loop trying to fetch location and preventing the device from sleeping. This should
  improve battery life in some cases.
- Fix crash when starting the app right after quitting it.
- Restart background service if it stops responding.
- Fix crash when VPN permission is revoked, either manually or by starting another VPN app.
- Fix crash caused by local JNI reference table overflow after running for a long time.
- Dismiss notification after service has stopped.
- Don't show missing connectivity error message in WireGuard key management screen if a
  reconnection is expected to happen.
- Fix showing new key as invalid immediately after regeneration.

#### Linux
- DNS management with static `/etc/resolv.conf` will now work even when no
  `/etc/resolv.conf` exists.

### Security
- Add automatic key rotation for WireGuard (every 7 days by default). This limits the potential
  for an attacker to correlate traffic with a public key and identity, and reduces the harm of
  software that might leak the private tunnel IP (since it is no longer fixed).

#### Windows
- Stop OpenVPN from loading `C:\etc\ssl\openssl.cnf` on start. This file was being loaded when an
  OpenVPN tunnel was being created. Any user could create the file, and the process loading it runs
  as the SYSTEM user. Since the config file allows loading arbitrary code, it was an attack vector
  allowing local unprivileged users to run code as SYSTEM.

#### macOS
- Limit macOS firewall rules to only allow UDP packets in the rules meant to enable being a DHCPv4
  *server* when local network sharing is enabled.


## [2019.10] - 2019-12-12
### Fixed
- Fix improved WireGuard port selection.

#### Windows
- Register 'NSI' service as a dependency of the daemon service.
- Set daemon service SID type as 'unrestricted'.
- Properly tear down routes after disconnecting from WireGuard relays.
- Fix bug that prohibited WireGuard from working over port 53.

### Security
#### Linux
- Stop [CVE-2019-14899](https://seclists.org/oss-sec/2019/q4/122) by dropping all packets destined
  for the tunnel IP coming in on some other interface than the tunnel.


## [2019.10-beta2] - 2019-12-05
### Added
- Add `mullvad relay set tunnel-protocol` subcommand to the CLI to specify what tunnel protocol
  to use.
- Add `mullvad reconnect` subcommand to the CLI to make the app pick a new server and reconnect.

#### Windows
- Full WireGuard support, GUI and CLI.
- Install Wintun driver that provides the WireGuard TUN adapter.
- Remove Mullvad TAP adapter on uninstall. Also remove the TAP driver if there are no other TAP
  adapters in the system.

#### Android
- Add connectivity status check. Stopping the app from sitting in a reconnect loop while the
  device is offline.

### Changed
- Notifications shown when connecting to a server include its location.
- Upgrade OpenVPN from 2.4.7 to 2.4.8.
- Upgrade OpenSSL from 1.1.1c to 1.1.1d.
- When using WireGuard without specifying a specific relay port, port 53 will be used after 2
  failed connection attempts for 2 out of 4 each successive connection attempts

#### Windows
- Use a larger icon in notifications on Windows 10.
- Only update DNS settings if updating would change the effective settings. This is a work-around
  to avoid invoking `netsh` unnecessarily and getting stuck in associated hangs.
- Don't restart the service immediately if it aborts several times in a row. Leave a window of ten
  minutes to allow for addressing the issue.
- Upgrade libsodium from 1.0.17 to 1.0.18.
- Upgrade NDIS 6 TAP driver from 9.21.2 to 9.24.2.

### Fixed
#### Linux
- Improve stability on Linux by using the routing netlink socket in its own thread.
- When trying to use `resolvconf` for managing DNS, the daemon will check if
  `dnsmasq` is running and misconfigured.
- Improve stability on Linux by simplifying route management code.

#### Windows
- Detect removal of the OpenVPN TAP adapter on reconnection attempts.
- Improve robustness in path environment variable logic in Windows installer. Handle the case
  where the registry value type is incorrectly set to be a regular string rather than an expandable
  string.
- Fix suspend and resume issues with OpenVPN by upgrading the TAP driver.
- Minor adjustment in online/offline detection logic. This change addresses misbehaving drivers
  that report the adapter flags incorrectly.

#### Android
- Don't try to fetch location when the app knows that it has no connectivity. This should reduce
  wake-ups (improving battery life) and also fix very large log files consuming storage space.
- Fix crash when a new version event is received while the app is in the main screen.

### Security
- Force OpenVPN to use TLS 1.2 or newer, and limit the TLS 1.3 ciphers to only the strongest ones.
  The Mullvad servers have never allowed any insecure ciphers, so this was not really a problem.
  Just one extra safety precaution.


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
- Settings format updated to `v2`.

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
