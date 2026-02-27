# Logging and telemetry

The primary focus of the Mullvad VPN app is of course the users' privacy and anonymity.
For the purpose of debugging issues and in order to alert users who might be at risk, some
logging and minimal telemetry is being performed.


## Logging

For debugging and support purposes, the app's system service and GUI writes logs on the *local
device*. These logs are readable by all users on the system, but never automatically sent
anywhere by the app. Logs are sent to Mullvad only by explicitly sending a problem report,
see below.

The paths to the logs can be found in the main [README](../README.md).

The app must not log the Mullvad account number, device id, device name or WireGuard key material.

On Windows, a crashdump named `DAEMON.DMP` is being generated when `mullvad-daemon.exe` crashes.
It is never sent anywhere, but stored locally in the same directory as the other logs
if the user/a developer would like to investigate the crash.

### Problem reports

Reporting issues with the app to Mullvad's support is opt-in and manual. The app
never sends any logs or crash dumps to Mullvad by itself. A user needs to do this
explicitly by going to Settings -> Report a problem or by using the
`mullvad-problem-report` CLI tool (desktop only).

The logs collected for problem reports are redacted before sent, and the user
always has the option to see exactly what information is going to be submitted.
The following is redacted:

* Anything looking like a Mullvad account number - They are not logged to begin
  with. But just to be extra sure, any 16 digit number is also redacted.
* Home directory - In order to avoid including the current user's username in
  the logs.
* IPs and MAC addresses.
* V4 UUIDs. This includes account and device IDs, and network interface GUIDs on Windows.

Just like all other API communication, the problem reports are sent encrypted (TLS) with
server certificate pinning.


## Telemetry (version check)

<!--
This section of the docs is an *explanation*, and below it comes a *reference*. Please try
to follow the documentation guidelines on this in https://github.com/mullvad/coding-guidelines/
-->

The app reports a very minimal amount of telemetry to Mullvad. And it does not in any way tie
it to identifiable information. See reference below for exact telemetry data.

The app calls an API designed to tell the app if there are any upgrades available and
if the currently running version is still supported. The main purpose
is to inform the users about new app versions, and alert the user if there are known
vulnerabilities or bugs in the version they are currently running. All of this is first and
foremost to improve the user experience and keep the user safe.

This API request does not contain the currently used Mullvad account number or any other Mullvad
device identifier. It only contains which version of the app is currently running and which
operating system version it's running on.
The API server aggregates this information and only keeps counters on number of used app versions
and operating systems. These statistics are recorded for the purpose of letting Mullvad
understand the impact of discovered bugs and issues, and to prioritize features.

### Reference

The following is the telemetry included in the version check API call. These are sent as
http headers and are only submitted once per 24 hours:

* `M-App-Version`: Contains the version of the Mullvad VPN app. For example `2026.1`.
* `M-Platform-Version`: Contains the operating system name and version. Only the most important
  parts of the OS version number is included. It will never include patch versions or build numbers.
  Examples: `Windows 11`, `Linux Ubuntu 24.04`, `macOS 26.0`, `Android 16`.
