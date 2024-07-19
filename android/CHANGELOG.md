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
- Add the ability to customize how the app talks to the api.
- Add auto connect and start setting for devices without system vpn settings.

### Changed
- Migrate underlaying communication wtih daemon to gRPC. This also implies major changes and
  improvements throughout the app.

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
