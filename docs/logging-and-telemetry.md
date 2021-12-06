# Logging and telemetry

The primary focus of the Mullvad VPN app is of course the users' privacy and anonymity.
For the purpose of debugging issues and in order to alert users who might be at risk, some
logging and minimal telemetry is being performed.


## Logging

For debugging and support purposes, the app's system service and GUI writes logs on the *local
device*. These stay on the user's device unless the user explicitly sends a problem report,
see below.

The paths to the logs can be found in the main [README](../README.md)

The app must not log the Mullvad account number, device id, device name or WireGuard key material.

### Problem reports

Reporting issues with the app to Mullvad's support is opt-in and manual. The app
never sends any logs or crash dumps to Mullvad by itself. A user needs to do this
explicitly by going to Settings -> Report a problem or by using the
`mullvad-problem-report` CLI tool (desktop only).

The logs collected for problem reports are redacted before sent, and the user
always has the option to see exactly what information is going to be submitted.
The following is being redacted:

* Anything looking like a Mullvad account number - They are not logged to begin
  with. But just to be extra sure, any 16 digit number is also redacted.
* Home directory - In order to avoid including the current user's username in
  the logs.
* IPs and MAC addresses.
* GUIDs.

Just like all other API communication, the problem reports are sent encrypted (TLS) with
server certificate pinning.


## Telemetry

The app collects a very minimal amount of telemetry, and it does not in any way tie
it to an account number, IP or other identifiable information. The *only* telemetry
performed are aggregate numbers on which app versions are used and which operating system
versions they are used on. This is referred to as the "version check".

Every 24 hours the app calls an API designed to tell the app if there are any
upgrades available and if the currently running version is still supported. This will
alert the user if there are known vulnerabilities or if this version of the app is
known to malfunction. All of this is first and foremost to improve
the user experience and keep the user safe. This API call does not contain the currently
used Mullvad account number or any other Mullvad device identifier. It only contains which
version of the app is currently running and which operating system version it's running on.
The API server aggregate this information and only keeps counters on used app versions
and operating systems. These statistics are recorded so that the development team can
understand the impact of discovered bugs and issues, and to prioritize features.

Just like all other API communication, the version checks are sent encrypted (TLS) with
server certificate pinning.

### Operating system version

Only the most important parts of the OS version number is included.
This means it can be `Windows 10`, `Linux Ubuntu 20.04`, `macOS 12.0` or similar.
But not more granular than that. It will never include patch versions or build numbers.