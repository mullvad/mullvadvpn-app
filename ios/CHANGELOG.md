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
- Add option to block gambling and adult content.
- Add last used account field to login view.
- Display device name under account view.
- Add revoked device view displayed when the app detects that device is no longer registered on
  backend.
- Add ability to manage registered devices if too many devices detected during log-in.
- Add continuous monitoring of tunnel connection. Verify ping replies to detect whether traffic is 
  really flowing.
- Check if device is revoked or account has expired when the tunnel fails to connect on each second
  failed attempt.

### Changed
- When logged into an account with no time left, a new view is shown instead of account settings,
with the option to buy more time.
- Use exponential backoff with jitter for delay interval when retrying REST API requests.
- REST API requests will bypass VPN when tunnel is not functional.

### Fixed
- Improve random port distribution. Should be less biased towards port 53.
- Fix invalid map camera position during the app launch and keep it up to date when multitasking.
- Fix animation glitch when expanding partially visible cell in location picker.
- Periodically refresh account expiry in-app notification.

## Removed
- Remove iOS 12 support.


## [2022.2] - 2022-04-28
### Added
- Add tunnel monitor when establishing tunnel connection. Picks next relay every 15 seconds until
  any inbound traffic received. This should also keep the tunnel in connecting or reconnecting state
  until the tunnel monitor determined that connection is functional.
- Add "FAQ & Guides" link in Settings.

### Changed
- Delete leftover settings in Keychain during login. WireGuard keys will be removed from
  server too if old settings can be read. This is usually the case when uninstalling the app and
  then reinstalling it without logging out first.
- Validate account token before charging user (in-app purchases). Safeguards from trying to add
  credits on accounts that no longer exist on our backend. Usually the case with newly created
  accounts that went stale.


## [2022.1] - 2022-02-15
### Added
- Show privacy overlay when entering app switcher.
- Add option to block malware.

### Fixed
- Fix crash occurring after completing in-app purchase.
- Fix error when changing relays while in airplane mode.
- Prevent key rotation from clogging the server key list by storing the next key and reusing it
  until receiving the successful response from Mullvad API. Add up to three retry attempts.

### Changed
- Increase hit area of settings (cog) button.
- Update launch screen.
- Never use DNS to talk to Mullvad API. Instead use the list of IP addresses bundled with the app
  and update it periodically.


## [2021.4] - 2021-11-30
### Added
- Add ability to specify custom DNS servers.

### Changed
- Attach log backup from previous application run to problem report.
- Use background tasks to periodically update relays and rotate the private key on iOS 13 or newer.
  Background fetch is used as fallback on iOS 12.
- Request background execution time from the system when performing critical tasks.

### Fixed
- Drop leading replacement characters (`\u{FFFD}`) when decoding UTF-8 from a part of log file.

### Security
- Move REST API networking from the packet tunnel process to the main process to prevent leaking
  traffic outside of the tunnel.


## [2021.3] - 2021-08-10
### Added
- Show a reminder to add more credits 3 days before account expiry via system notification and
  in-app message.
- Add submit button next to account input field on login screen.

### Fixed
- Update WireGuardKit to the latest. Fixes iOS 15 support.
- Improve accessibility support.


## [2021.2] - 2021-06-03
### Added
- Enable option to "Select all" when viewing app logs.
- Split view interface for iPad.
- Add interactive map.
- Reduce network traffic consumption by leveraging HTTP caching via ETag HTTP header to avoid
  re-downloading the relay list if it hasn't changed.
- Pin root SSL certificates.
- Add option to use Mullvad's ad-blocking DNS servers.

### Fixed
- Fix bug which caused the tunnel manager to become unresponsive in the rare event of failure to
  disable on-demand when stopping the tunnel from within the app.
- Fix bug that caused the app to skip tunnel settings migration from older versions of the app.
- Localize some of well known StoreKit errors so that they look less cryptic when presented to user.
- Improve tunnel settings verification to address issues with broken tunnel and missing Keychain
  entries to tunnel settings in cases such as when setting up a new device from backup.


## [2021.1] - 2021-03-16
### Added
- Add ability to report a problem inside the app. Sends logs to support.

### Changed
- Migrate to WireGuardKit framework.

### Fixed
- Fix crash when pasting empty string into account input field.
- Fix invalid initial text color of "unsecured connection" label on iOS 12.


## [2020.5] - 2020-11-04
### Fixed
- Fix regression where "Internal error" was displayed instead of server error (i.e too many
  WireGuard keys)


## [2020.4] - 2020-09-10
### Added
- Save application logs to file.
- Add button to reconnect the tunnel.
- Add support for iOS 12.
- Ship the initial relay list with the app, and do once an hour periodic refresh in background.
- Refresh account expiry when visiting settings.

### Fixed
- Fix the issue when starting the tunnel could take longer than expected due to the app refreshing
  the relay list before connecting.
- Fix the issue when regenerating the WireGuard key and dismissing the settings at the same
  time could lead to the revoked key still being used by the tunnel, leaving the tunnel unusable.

### Changed
- Remove the WireGuard key from the account inside the VPN tunnel during the log out, if VPN is
  active at that time. Before it would always remove it outside the tunnel.
- Turn off WireGuard backend when there are no active network interfaces available. Saves battery.
- Switch from JSON-RPC to REST communication protocol when talking to Mullvad API servers.


## [2020.3] - 2020-06-12
### Added
- Add automatic key rotation every 4 days.

### Fixed
- Fix relay selection for country wide constraints by respecting the `include_in_country`
  parameter.
- Fix defect when manually regenerating the private key from Settings would automatically connect
  the tunnel.
- Properly format date intervals close to 1 day or less than 1 minute. Enforce intervals between 1
  and 90 days to always be displayed in days quantity.
- Fix a number of errors in DNS64 resolution and IPv6 support.
- Update the tunnel state when the app returns from suspended state.
- Disable `URLSession` cache. Fixes audit finding [`MUL-02-001`]

[`MUL-02-001`]: ../audits/2020-06-12-cure53.md#miscellaneous-issues


## [2020.2] - 2020-04-16
### Fixed
- Fix "invalid account" error that was mistakenly reported as "network error" during log in.
- Fix parsing of pre-formatted account numbers when pasting from pasteboard on login screen.

### Added
- Format account number in groups of 4 digits separated by whitespace on login screen.
- Enable on-demand VPN with a single rule to always connect the tunnel when on Wi-Fi or cellular.
  Automatically disable on-demand VPN when manually disconnecting the tunnel from GUI to prevent the
  tunnel from coming back up.


## [2020.1] - 2020-04-08
Initial release. Supports...
* Establishing WireGuard tunnels
* Selecting and changing location and servers
* See account expiry
* Purchase more VPN time via in-app purchases
* See the current WireGuard key in use and how long it has been used
* Generate a new WireGuard key to replace the old
