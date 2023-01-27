# Mullvad VPN desktop and mobile app

Welcome to the Mullvad VPN client app source code repository.
This is the VPN client software for the Mullvad VPN service.
For more information about the service, please visit our website,
[mullvad.net](https://mullvad.net) (Also accessible via Tor on our
[onion service](http://o54hon2e2vj6c7m3aqqu6uyece65by3vgoxxhlqlsvkmacw6a7m7kiad.onion/)).

This repository contains all the source code for the
desktop and mobile versions of the app. For desktop this includes the system service/daemon
([`mullvad-daemon`](mullvad-daemon/)), a graphical user interface ([GUI](gui/)) and a
command line interface ([CLI](mullvad-cli/)). The Android app uses the same backing
system service for the tunnel and security but has a dedicated frontend in [android/](android/).
iOS consists of a completely standalone implementation that resides in [ios/](ios/).

## Releases

There are built and signed releases for macOS, Windows, Linux and Android available on
[our website](https://mullvad.net/download/) and on
[Github](https://github.com/mullvad/mullvadvpn-app/releases/). The Android app is also available
on [Google Play] and [F-Droid] and the iOS version on [App Store].

[Google Play]: https://play.google.com/store/apps/details?id=net.mullvad.mullvadvpn
[F-Droid]: https://f-droid.org/packages/net.mullvad.mullvadvpn/
[App Store]: https://apps.apple.com/us/app/mullvad-vpn/id1488466513

You can find our code signing keys as well as instructions for how to cryptographically verify
your download on [Mullvad's Open Source page].

### Platform/OS support

These are the operating systems and their versions that the app officially supports. It might
work on many more versions, but we don't test for those and can't guarantee the quality or
security.

| OS/Platform | Supported versions |
|-------------|--------------------|
| Windows     | 10 and 11          |
| macOS       | The three latest major releases |
| Linux (Ubuntu)| The two latest LTS releases and the latest non-LTS releases |
| Linux (Fedora) | The versions that are not yet [EOL](https://fedoraproject.org/wiki/End_of_life) |
| Linux (Debian) | The versions that are not yet [EOL](https://wiki.debian.org/DebianReleases) |
| Android | The four latest major releases|
| iOS         | 12 and newer       |

On Linux we test using the Gnome desktop environment. The app should, and probably does work
in other DEs, but we don't regularly test those.

## Features

Here is a table containing the features of the app across platforms. This reflects the current
state of latest master, not necessarily any existing release.

|                               | Windows | Linux | macOS | Android | iOS |
|-------------------------------|:-------:|:-----:|:-----:|:-------:|:---:|
| OpenVPN                       |    ✓    |   ✓   |   ✓   |         |     |
| WireGuard                     |    ✓    |   ✓   |   ✓   |    ✓    |  ✓  |
| OpenVPN over Shadowsocks      |    ✓    |   ✓   |   ✓   |         |     |
| Split tunneling               |    ✓    |   ✓   |       |    ✓    |     |
| Custom DNS server             |    ✓    |   ✓   |   ✓   |    ✓    |  ✓  |
| Ad and tracker blocking       |    ✓    |   ✓   |   ✓   |         |  ✓  |
| Optional local network access |    ✓    |   ✓   |   ✓   |    ✓    |  ✓\* |

\* The local network is always accessible on iOS with the current implementation

## Security and anonymity

This app is a privacy preserving VPN client. As such it goes to great lengths to stop traffic
leaks. And basically all settings default to the more secure/private option. The user has to
explicitly allow more loose rules if desired. See the [dedicated security document] for details
on what the app blocks and allows, as well as how it does it.

[dedicated security document]: docs/security.md

## Checking out the code

This repository contains submodules needed for building the app. However, some of those submodules
also have further submodules that are quite large and not needed to build the app. So unless
you want the source code for OpenSSL, OpenVPN and a few other projects you should avoid a recursive
clone of the repository. Instead clone the repository normally and then get one level of submodules:
```bash
git clone https://github.com/mullvad/mullvadvpn-app.git
cd mullvadvpn-app
git submodule update --init
```

We sign every commit on the master branch as well as our release tags. If you would like to verify
your checkout, you can find our developer keys on [Mullvad's Open Source page].

### Binaries submodule

This repository has a git submodule at `dist-assets/binaries`. This submodule contains binaries and
build scripts for third party code we need to bundle with the app. Such as OpenVPN, Wintun
etc.

This submodule conforms to the same integrity/security standards as this repository. Every merge
commit should be signed. And this main repository should only ever point to a signed merge commit
of the binaries submodule.

See the [binaries submodule's](https://github.com/mullvad/mullvadvpn-app-binaries) README for more
details about that repository.

## Building the app

See the [build instructions](BuildInstructions.md) for help building the app on desktop platforms.

For building the Android app, see the [instructions](./android/BuildInstructions.md) for Android.

For building the iOS app, see the [instructions](./ios/BuildInstructions.md) for iOS.

## Releasing the app

See [this](Release.md) for instructions on how to make a new release.

## Environment variables used by the service

* `TALPID_FIREWALL_DEBUG` - Helps debugging the firewall. Does different things depending on
  platform:
  * Linux: Set to `"1"` to add packet counters to all firewall rules.
  * macOS: Makes rules log the packets they match to the `pflog0` interface.
    * Set to `"all"` to add logging to all rules.
    * Set to `"pass"` to add logging to rules allowing packets.
    * Set to `"drop"` to add logging to rules blocking packets.

* `TALPID_FIREWALL_DONT_SET_SRC_VALID_MARK` - Forces the daemon to not set `src_valid_mark` config
    on Linux. The kernel config option is set because otherwise strict reverse path filtering may
    prevent relay traffic from reaching the daemon. If `rp_filter` is set to `1` on the interface
    that will be receiving relay traffic, and `src_valid_mark` is not set to `1`, the daemon will
    not be able to receive relay traffic.

* `TALPID_DNS_MODULE` - Allows changing the method that will be used for DNS configuration.
  By default this is automatically detected, but you can set it to one of the options below to
  choose a specific method.

  * Linux
    * `"static-file"`: change the `/etc/resolv.conf` file directly
    * `"resolvconf"`: use the `resolvconf` program
    * `"systemd"`: use systemd's `resolved` service through DBus
    * `"network-manager"`: use `NetworkManager` service through DBus

  * Windows
    * `netsh`: use the `netsh` program
    * `tcpip`: set TCP/IP parameters in the registry

* `TALPID_FORCE_USERSPACE_WIREGUARD` - Forces the daemon to use the userspace implementation of
   WireGuard on Linux.

* `TALPID_DISABLE_OFFLINE_MONITOR` - Forces the daemon to always assume the host is online.

* `TALPID_NET_CLS_MOUNT_DIR` - On Linux, forces the daemon to mount the `net_cls` controller in the
  specified directory if it isn't mounted already.

* `MULLVAD_MANAGEMENT_SOCKET_GROUP` - On Linux and macOS, this restricts access to the management
  interface UDS socket to users in the specified group. This means that only users in that group can
  use the CLI and GUI. By default, everyone has access to the socket.

### Development builds only

* `MULLVAD_API_HOST` - Set the hostname to use in API requests. E.g. `api.mullvad.net`.

* `MULLVAD_API_ADDR` - Set the IP address and port to use in API requests. E.g. `10.10.1.2:443`.

* `MULLVAD_API_DISABLE_TLS` - Use plain HTTP for API requests.

### Setting environment variables

#### Windows

Use `setx` from an elevated shell:

```bat
setx TALPID_DISABLE_OFFLINE 1 /m
```

For the change to take effect, restart the daemon:

```bat
sc.exe stop mullvadvpn
sc.exe start mullvadvpn
```

#### Linux

Edit the systemd unit file via `systemctl edit mullvad-daemon.service`:

```ini
[Service]
Environment="TALPID_DISABLE_OFFLINE_MONITOR=1"
```

For the change to take effect, restart the daemon:

```bash
sudo systemctl restart mullvad-daemon
```

#### macOS

Use `launchctl`:

```bash
sudo launchctl setenv TALPID_DISABLE_OFFLINE_MONITOR 1
```

For the change to take effect, restart the daemon:

```bash
launchctl unload -w /Library/LaunchDaemons/net.mullvad.daemon.plist
launchctl load -w /Library/LaunchDaemons/net.mullvad.daemon.plist
```

## Environment variables used by the GUI frontend

* `MULLVAD_PATH` - Allows changing the path to the folder with the `mullvad-problem-report` tool
   when running in development mode. Defaults to: `<repo>/target/debug/`.
* `MULLVAD_DISABLE_UPDATE_NOTIFICATION` - If set to `1`, GUI notification will be disabled when
   an update is available.


## Command line tools for Electron GUI app development

- `$ npm run develop` - develop app with live-reload enabled
- `$ npm run lint` - lint code
- `$ npm run pack:<OS>` - prepare app for distribution for your platform. Where `<OS>` can be
  `linux`, `mac` or `win`
- `$ npm test` - run tests


## Tray icon on Linux

The requirements for displaying a tray icon varies between different desktop environments. If the
tray icon doesn't appear, try installing one of these packages:
- `libappindicator3-1`
- `libappindicator1`
- `libappindicator`

If you're using GNOME, try installing one of these GNOME Shell extensions:
- `TopIconsFix`
- `TopIcons Plus`


## Repository structure

### Electron GUI app and electron-builder packaging assets
- **gui/**
  - **assets/** - Graphical assets and stylesheets
  - **src/**
    - **main/**
      - **index.ts** - Entry file for the main process
    - **renderer/**
      - **app.tsx** - Entry file for the renderer process
      - **routes.tsx** - Routes configurator
      - **transitions.ts** - Transition rules between views
    - **config.json** - App color definitions and URLs to external resources
  - **tasks/** - Gulp tasks used to build app and watch for changes during development
    - **distribution.js** - Configuration for `electron-builder`
  - **test/** - Electron GUI tests
- **dist-assets/** - Icons, binaries and other files used when creating the distributables
  - **binaries/** - Git submodule containing binaries bundled with the app. For example the
    statically linked OpenVPN binary. See the README in the submodule for details
  - **linux/** - Scripts and configuration files for the deb and rpm artifacts
  - **pkg-scripts/** - Scripts bundled with and executed by the macOS pkg installer
  - **windows/** - Windows NSIS installer configuration and assets
  - **ca.crt** - The Mullvad relay server root CA. Bundled with the app and only OpenVPN relays
    signed by this CA are trusted


### Building, testing and misc
- **build-windows-modules.sh** - Compiles the C++ libraries needed on Windows
- **build.sh** - Sanity checks the working directory state and then builds installers for the app

### Mullvad Daemon

The daemon is implemented in Rust and is implemented in several crates. The main, or top level,
crate that builds the final daemon binary is `mullvad-daemon` which then depend on the others.

In general one can look at the daemon as split into two parts, the crates starting with `talpid`
and the crates starting with `mullvad`. The `talpid` crates are supposed to be completely unrelated
to Mullvad specific things. A `talpid` crate is not allowed to know anything about the API through
which the daemon fetch Mullvad account details or download VPN server lists for example. The
`talpid` components should be viewed as a generic VPN client with extra privacy and anonymity
preserving features. The crates having `mullvad` in their name on the other hand make use of the
`talpid` components to build a secure and Mullvad specific VPN client.


- **Cargo.toml** - Main Rust workspace definition. See this file for which folders here are daemon
  Rust crates.
- **mullvad-daemon/** - Main Rust crate building the daemon binary.
- **talpid-core/** - Main crate of the VPN client implementation itself. Completely Mullvad agnostic
  privacy preserving VPN client library.


## Vocabulary

Explanations for some common words used in the documentation and code in this repository.

- **App** - This entire product (everything in this repository) is the "Mullvad VPN App", or App for
  short.
  - **Daemon** - Refers to the `mullvad-daemon` Rust program. This headless program exposes a
    management interface that can be used to control the daemon
  - **Frontend** - Term used for any program or component that connects to the daemon management
    interface and allows a user to control the daemon.
    - **GUI** - The Electron + React program that is a graphical frontend for the Mullvad VPN App.
    - **CLI** - The Rust program named `mullvad` that is a terminal based frontend for the Mullvad
      VPN app.


## File paths used by Mullvad VPN app

A list of file paths written to and read from by the various components of the Mullvad VPN app

### Daemon

On Windows, when a process runs as a system service the variable `%LOCALAPPDATA%` expands to
`C:\Windows\system32\config\systemprofile\AppData\Local`.

All directory paths are defined in, and fetched from, the `mullvad-paths` crate.

#### Settings

The settings directory can be changed by setting the `MULLVAD_SETTINGS_DIR` environment variable.

| Platform | Path |
|----------|------|
| Linux | `/etc/mullvad-vpn/` |
| macOS | `/etc/mullvad-vpn/` |
| Windows | `%LOCALAPPDATA%\Mullvad VPN\` |
| Android | `/data/data/net.mullvad.mullvadvpn/` |

#### Logs

The log directory can be changed by setting the `MULLVAD_LOG_DIR` environment variable.

| Platform | Path |
|----------|------|
| Linux | `/var/log/mullvad-vpn/` + systemd |
| macOS | `/var/log/mullvad-vpn/` |
| Windows | `C:\ProgramData\Mullvad VPN\` |
| Android | `/data/data/net.mullvad.mullvadvpn/` |

#### Cache

The cache directory can be changed by setting the `MULLVAD_CACHE_DIR` environment variable.

| Platform | Path |
|----------|------|
| Linux | `/var/cache/mullvad-vpn/` |
| macOS | `/Library/Caches/mullvad-vpn/` |
| Windows | `C:\ProgramData\Mullvad VPN\cache` |
| Android | `/data/data/net.mullvad.mullvadvpn/cache` |

#### RPC address file

The full path to the RPC address file can be changed by setting the `MULLVAD_RPC_SOCKET_PATH`
environment variable.

| Platform | Path |
|----------|------|
| Linux | `/var/run/mullvad-vpn` |
| macOS | `/var/run/mullvad-vpn` |
| Windows | `//./pipe/Mullvad VPN` |
| Android | `/data/data/net.mullvad.mullvadvpn/rpc-socket` |

### GUI

The GUI has a specific settings file that is configured for each user. The path is set in the
`gui/packages/desktop/main/gui-settings.ts` file.

| Platform | Path |
|----------|------|
| Linux | `$XDG_CONFIG_HOME/Mullvad VPN/gui_settings.json` |
| macOS | `~/Library/Application Support/Mullvad VPN/gui_settings.json` |
| Windows | `%LOCALAPPDATA%\Mullvad VPN\gui_settings.json` |
| Android | Present in Android's `logcat` |

## Icons

See [graphics README](graphics/README.md) for information about icons.

## Locales and translations

Instructions for how to handle locales and translations are found
[here](./gui/locales/README.md).

For instructions specific to the Android app, see [here](./android/README.md).

## Audits, pentests and external security reviews

Mullvad has used external pentesting companies to carry out security audits of this VPN app. Read
more about them in the [audits readme](./audits/README.md).

# License

Copyright (C) 2022  Mullvad VPN AB

This program is free software: you can redistribute it and/or modify it under the terms of the
GNU General Public License as published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.

For the full license agreement, see the LICENSE.md file

The source code for the iOS app is GPL-3 licensed like everything else in this repository.
But the distributed app on the Apple App Store is not GPL licensed,
it falls under the [Apple App Store EULA].

[Apple App Store EULA]: https://www.apple.com/legal/internet-services/itunes/dev/stdeula/
[Mullvad's Open Source page]: https://mullvad.net/en/guides/open-source/
