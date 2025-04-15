# Android changelog
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
- Make feature indicators clickable, allowing for easy access to active features.


## [android/2025.2-beta2] - 2025-04-15
### Fixed
- Fix focus on TV devices when returning to connect screen from select location.


## [android/2025.2-beta1] - 2025-04-04
### Added
- Prompt password manager to store new account number on account creation.
- Add the ability to force the ip version used to connect to a relay.
- Add the ability to disable IPv6 in the tunnel.

### Changed
- Disable Wireguard port setting when a obfuscation is selected since it is not used when an
  obfuscation is applied.
- Adapt UI on Connect Screen for Android TV, including a navigation rail and redesigned in-app
  notification bar.

### Removed
- Remove Google's resolvers from encrypted DNS proxy.

### Fixed
- Will no longer try to connect using an IP version if that IP version is not available.
- Fix connection details showing in IP from exit server instead of entry when using multihop.


## [android/2025.1] - 2025-03-20
Identical to `android/2025.1-beta1`


## [android/2025.1-beta1] - 2025-03-05
### Fixed
- Fix a crash that could occur in the Filter screen.
- Fix a bug that could cause the app to crash while navigating.

### Security
- Make daemon aware of route changes to prevent sending traffic before routes are up.
- Minimize calls to re-establish the VPN tunnel, since this may cause Android to leak some traffic.


## [android/2024.10-beta2] - 2024-12-20
### Fixed
- Update bundled relay list to address a UI bug in the filter screen.


## [android/2024.10-beta1] - 2024-12-19
### Added
- Add multihop which allows the routing of traffic through an entry and exit server, making it
  harder to trace.
- Enable DAITA to route traffic through servers with DAITA support to enable the use
  of all servers together with DAITA. This behaviour can be disabled with the use of the
  "Direct only" setting.

### Changed
- Update to DAITA v2. The main difference is that many different machines are provided by relays
  instead of a bundled list.

### Fixed
- Fix a crash that would occur because a Provider would be listed twice in the filter screen.


## [android/2024.9] - 2024-12-09
### Changed
- Improve detection and logging of a potential rare in-app purchase limbo state.


## [android/2024.9-beta1] - 2024-11-27
### Added
- Add a new access method: Encrypted DNS Proxy. Encrypted DNS proxy is a way to reach the API via
  proxies. The access method is enabled by default.

### Changed
- Animation has been changed to look better with predictive back.

### Fixed
- Fix a bug where the Android account expiry notifications would not be updated if the app was
  running in the background for a long time.
- Fix ANR due to the tokio runtime being blocked by `getaddrinfo` when dropped.
- Fix crash when having a legacy VPN profile as always-on.

### Security
- Remove alternative stack for fault signal handlers on unix based systems. It was implemented
  incorrectly and could cause stack overflow and heap memory corruption.
  Fixes audit issue `MLLVD-CR-24-01`.
- Remove/disable unsafe signal code from fault signal handler on unix based systems.
  Fixes audit issue `MLLVD-CR-24-02`.


## [android/2024.8] - 2024-11-01
### Fixed
- Fix the account number input keyboard being broken on Amazon FireStick by adding a workaround.
  This should eventually be fixed by Amazon since the FireStick behavior is broken.


## [android/2024.8-beta2] - 2024-10-25
### Fixed
- Improve connection stability when roaming while using Shadowsocks.
- Fix MTU calculation to avoid connectivity issues when using some specific settings.


## [android/2024.8-beta1] - 2024-10-21
### Added
- Add feature indicators to the main view along with redesigning the connection details.
- Add new "Connect on device start-up" setting for devices without system VPN settings.
- Add a confirmation dialog shown when creating a new account if there's already an existing
  account in the account history of the login screen.

### Changed
- Replace the draft key encapsulation mechanism Kyber (round 3) with the standardized
  ML-KEM (FIPS 203) dito in the handshake for Quantum-resistant tunnels.
- Move version information and changelog to a new app info screen.
- Update icons to material design.

### Fixed
- Fix unlabeled icon buttons for basic accessibility with screen readers.


## [android/2024.7] - 2024-10-18
### Fixed
- Fix a bug where tunnel obfuscation (UDP-over-TCP or Shadowsocks) only worked in combination with
  either DAITA or quantum-resistant tunnels, but only after the initial tunnel negotiation used for
  both DAITA and quantum-resistant tunnels. This combination of issues made the obfuscation methods
  effectively unusable behind restrictive firewalls regardless of setting combination.


## [android/2024.6] - 2024-10-14
### Fixed
- Fix rare crash related to an upcoming feature (feature indicators).


## [android/2024.5] - 2024-10-09
### Fixed
- Fix crash when in the edit custom list locations screen and changing app langauge.


## [android/2024.5-beta2] - 2024-09-24
### Fixed
- Fix building of the app bundle which is used for publishing to Google Play.


## [android/2024.5-beta1] - 2024-09-23
### Added
- Add support for predictive back.
- Add DAITA (Defence against AI-guided Traffic Analysis) setting.
- Add WireGuard-over-Shadowsocks.

### Changed
- Update colors in the app to be more in line with material design.

### Fixed
- Fix VPN service being recreated multiple times when toggling certain options.
- Fix location selection navigation on some TV devices.


## [android/2024.4] - 2024-09-03
### Fixed
- Fix potential crash when toggling quick settings tile.
- Fix crash when starting and stopping service too fast.
- Fix crashes on TV.


## [android/2024.4-beta2] - 2024-08-07
### Fixed
- Fix and improve the gRPC communication.
- Fix stuck in splash screen.
- Fix privacy policy crash occurring in some usage flows.
- Fix tunnel state notification sometimes showing incorrect information when logged out.


## [android/2024.4-beta1] - 2024-07-26
### Added
- Add API access setting which makes it possible to configure how the app communicates with our API.
- Add splash screen animation.

### Changed
- Migrate underlying communication with daemon to gRPC. This also implies major changes and
  improvements throughout the app.

### Fixed
- Fix animations in the location selection screen.


## [android/2024.3] - 2024-06-27
Identical to `android/2024.3-beta1` except for updated translations.


## [android/2024.3-beta1] - 2024-06-14
### Changed
- Always show account history if present on login screen.
- Clarifications to in-app lockdown guide.

### Deprecated
- Auto-connect is now legacy on all android devices. This setting will be replaced with a new one
  for devices without native OS support.

### Fixed
- Improve device revoked detection.


## [android/2024.2] - 2024-05-29
### Fixed
- Remove potentially sensitive tunnel state information from lifecycle log.


## [android/2024.2-beta2] - 2024-05-08
### Security
- Fix DNS leaks in blocking states or when no valid DNS has been configured due to an underlying OS
  issue. In these cases a dummy DNS will be set to prevent leaks.
- Clarify lockdown limitations in the in-app guide.


## [android/2024.2-beta1] - 2024-04-17
### Added
- Add the ability to create and manage custom lists of relays.
- Add the ability to import server IP overrides using text or file.

### Changed
- Change [default retry connection attempts][`relay selector defaults`].

[`relay selector defaults`]: docs/relay-selector.md#default-constraints-for-tunnel-endpoints

### Fixed
- Fix pointless API access method rotations for concurrent requests.
- Fix broken IPv6 connectivity by making sure the relay selector attempts IPv6 connections.


## [android/2024.1] - 2024-04-05
### Fixed
- Fix 3D map animation distance calculation.


## [android/2024.1-beta1] - 2024-03-18
### Added
- Add 3D map to the connect screen.
- Add support for all screen orientations.
- Add possibility to filter locations by provider.
- Add toggle for enabling or disabling split tunneling.
- Add auto-connect and lockdown guide on platforms with system vpn settings.

### Changed
- Migrate to Compose Navigation which also improves screen transition animations.
- Increase focus highlight opacity.
- Set auto-connect setting as legacy on platforms with system vpn settings.
- Change default obfuscation setting to `auto`.
- Migrate obfuscation settings for existing users from `off` to `auto`.
- Update support email address to new email address, support@mullvadvpn.net.

### Fixed
- Improve DPAD navigation.
- Upgrade wireguard-go. This might improve connectivity on some devices such as chromebooks.
- Fix connectivity issues that would occur when using quantum-resistant tunnels with an incorrectly
  configured MTU.

### Security
- Change from singleTask to singleInstance to fix Task Affinity Vulnerability in Android 8.
- Add protection against some tapjacking vulnerabilities.


## [android/2023.10] - 2023-12-14
Identical to `android/2023.10-beta1`.


## [android/2023.10-beta1] - 2023-12-11
### Fixed
- Fix relay selector attempting to connect to OpenVPN relays in some circumstances.


## [android/2023.9] - 2023-12-06
### Added
- Add missing translations for in-app purchases and a few settings.


## [android/2023.8] - 2023-11-28
Identical to `android/2023.8-beta2`.


## [android/2023.8-beta2] - 2023-11-27
### Fixed
- Fix top bar flickering in some scrollable views.
- Fix welcome screen sometimes showing on app restart after adding time.
- Fix inconsistencies with the account history in the login view.
- Fix OS crash when sharing long logs by instead sharing the log content as a file.
- Improve in-app purchase and verification flow in some circumstances.


## [android/2023.8-beta1] - 2023-11-20
### Changed
- Add Google Play in-app purchases to the build distributed via Google Play.
- Add social media content blocker.
- Add support for setting per-app language in system settings.
- Improve device and expiry information throughout the app.
- Migrate remaining views to Compose and MVVM (welcome, out-of-time, login, problem report, logs
  voucher dialog, in-app notifications).
- Add share button to the log view which can be used to copy or in other ways share the log text.
  This was partially added to due to limitations in Compose which result in it not being possible to
  select and copy text in the log view.


## [android/2023.7] - 2023-10-11
### Changed
- Minor addition to problem report logs to aid debugging of user issues.


## [android/2023.6] - 2023-09-25
### Fixed
- Fix inconsistent dialog corner radius.
- Fix missing scrolling in the changes dialog.
- Fix unused bundled relay list.


## [android/2023.6-beta2] - 2023-09-13
### Fixed
- Fix tunnel state and connection details sometimes getting stuck showing the wrong information.
- Fix MTU dismiss behavior.
- Fix DNS input crash.
- Fix inconsistent dialog padding.


## [android/2023.6-beta1] - 2023-08-29
### Added
- Add quantum resistant tunneling.
- Add UDP-over-TCP WireGuard obfuscation.
- Improve how the Android firewall handles incoming connections on Android 11+ devices.
- Add search bar to the Select location view.
- Add settings entry to configure WireGuard port by either using a predefined or custom port.

### Changed
- Combine the "Preferences" and "Account" settings sub-menus into a single one called
  "VPN Settings".
- Make "Split tunneling" more accessible by placing it directly in the main settings menu.
- Migrate multiple views to Compose and MVVM (Settings, Account, Split tunneling, Select location).

### Fixed
- Reduce flickering in the main/connect view.


## [android/2023.5] - 2023-08-02
### Changed
- New fancy version number in order to try to resolve Google Play distribution issues. Otherwise
  same as `android/2023.4`.


## [android/2023.4] - 2023-07-18
### Changed
- Prevent opening download page in Google Play builds.


## [android/2023.3] - 2023-06-27
### Changed
- Change so that all links and texts leading to the mullvad webpage display a modified version of
  the webpage that does not include links to the account page in order to comply with
  the Google Play payment policies. This doesn't apply to F-Droid builds.
- Hide the FAQs and Guides button for Google Play users.


## [android/2023.2] - 2023-05-22
### Changed
- Change so that all links and texts leading to the account web page (which also includes a payment
  flow) are either hidden or leads to the app itself (notification actions) in order to comply with
  the Google Play payment policies. This doesn't apply to F-Droid builds.


## [android/2023.1] - 2023-05-16
### Fixed
- Fix DNS input keyboard type.


## [android/2023.1-beta2] - 2023-05-09
### Added
- Add "Manage account" button to the account view.

### Fixed
- Fix missing payment info in out-of-time view.


## [android/2023.1-beta1] - 2023-05-03
### Added
- Add themed icon.
- Add DNS content blockers.

### Changed
- Clarify some of the error messages throughout the app.
- Increase WireGuard key rotation interval to 14 days.
- Change the DNS/MTU input to rely on dialogs in order to improve the UX on some devices.
- Hide "Buy more credit" buttons in the default release build published to Google Play, our website
  and GitHub. The buttons are still visible for F-Droid builds.

### Fixed
- Fix adaptive app icon which previously had a displaced nose and some other oddities.
- Fix app version sometimes missing in the settings menu.


## [android/2022.3] - 2022-11-14
### Added
- Add privacy policy link in settings.
- Add initial privacy consent which is showed on each start until approved.


## [android/2022.2] - 2022-10-17
Identical to android/2022.2-beta2 except for updated translations.


## [android/2022.2-beta2] - 2022-09-09
### Changed
- Refresh device data when opening the account view to ensure the local data is up-to-date and that
  the device hasn't been revoked.
- Disable settings button during login.

### Fixed
- Fix crash sometimes occurring during account creation.
- Fix tunnel info expansion state not remembered during pause and resume.
- Fix crash during some view transitions.
- Fix disabled login button on login failure. Instead, the login button will now still be enabled
  on login failures to let the user re-attempt the login.


## [android/2022.2-beta1] - 2022-08-11
### Added
- Add device management to the Android app. This simplifies knowing which device is which and adds
  the option to log other devices out when the account already has five devices.

### Changed
- Lowered default MTU to 1280 on Android.
- Disable app icon badge for tunnel state notification/status.

### Removed
- Remove WireGuard view as it's no longer needed with the new way of managing devices.

### Fixed
- Fix unused dependencies loaded in the service/tile DI graph.
- Fix missing IPC message unregistration causing multiple copies of some messages to be received.
- Fix quick settings tile being unresponsive and causing crashes on some devices.
- Fix quick settings tile not working when the device is locked. It will now prompt the user to
  unlock the device before attempting to toggle the tunnel state.
- Fix crash when clicking in-app URL notifications.

### Security
- Prevent location request responses from being received outside the tunnel when in the connected
  state.


## [android/2022.1] - 2022-03-01
Identical to android/2022.1-beta3 except for a few updated translations.


## [android/2022.1-beta3] - 2022-02-08
### Fixed
- Fix app crash caused by quick settings tile.


## [android/2022.1-beta2] - 2022-01-27
### Fixed
- Fix app sometimes crashing during startup on Android TVs.


## [android/2022.1-beta1] - 2022-01-26
### Added
- Add toggle for Split tunneling view to be able to show system apps
- Add support of adaptive icons (available only from Android 8).

### Changed
- Gradually increase the WireGuard connectivity check timeout, lowering the timeout for the first
  few attempts.
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


## [android/2021.1] - 2021-05-04
This release is for Android only.

This release is identical to android/2021.1-beta1.
This is our first non beta release for the Android platform!


## [android/2021.1-beta1] - 2021-04-06
This release is for Android only. From now on, Android releases will have this new header format
that is the same as the git tag they receive: `android/<version>`.

### Added
- Add 51820 to list of WireGuard ports in app settings.
- Add option to connect to WireGuard relays over IPv6.
- Add Burmese translations.
- Allow reaching the API server when connecting, disconnecting or in a blocked state.
- Add FAQs & Guides menu entry to the Settings screen.
- Add TV banner for better user experience and requirements.
- Style StatucBar and NavigationBar to make our app a bit more beautiful.

### Changed
- Allow provider constraint to specify multiple hosting providers.
- Only download a new relay list if it has been modified.
- Connect to the API only via TLS 1.3
- Shrink account history capactity from 3 account entries to 1.
- WireGuard key is now rotated sooner: every four days instead of seven.

### Fixed
- Improve responsiveness when reconnecting after some failed connection attempts.
- Fix input area sometimes disappearing when returning to the Login screen.

For older non-stable releases, see the main changelog ../CHANGELOG.md
