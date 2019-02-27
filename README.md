# Mullvad VPN desktop and mobile app

The system service/daemon, GUI and CLI for the Mullvad VPN app.

## Releases

There are built and signed releases for macOS, Windows and Linux available on
[our website](https://mullvad.net/download/) and on
[Github](https://github.com/mullvad/mullvadvpn-app/releases/).
Support for Android and iOS is in the making.

You can find our code signing keys as well as instructions for how to cryptographically verify
your download on [Mullvad's Open Source page].

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

## Install toolchains and dependencies

Follow the instructions for your platform, and then the [All platforms](#all-platforms)
instructions.

These instructions are probably not complete. If you find something more that needs installing
on your platform please submit an issue or a pull request.

### Windows

The host has to have the following installed:

- Microsoft's _Build Tools for Visual Studio 2017_ (a regular installation of Visual Studio 2017
  Community edition works as well).

- Windows SDK *10.0.16299.0* (This exact version required)

- `msbuild.exe` available in `%PATH%`. If you installed Visual Studio Community edition, the
  binary can be found under:
  ```
  C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\MSBuild\<VERSION>\Bin\amd64
  ```

- `bash` installed as well as a few base unix utilities, including `sed` and `tail`.
  The environment coming with [Git for Windows] works fine.

[Git for Windows]: https://git-scm.com/download/win

### Linux

#### Debian/Ubuntu
```bash
# For building the daemon
sudo apt install gcc libdbus-1-dev
# For running the frontend app
sudo apt install gconf2
# For building the installer
sudo apt install rpm
```

#### Fedora/RHEL
```bash
# For building the daemon
sudo dnf install dbus-devel
# For building the installer
sudo dnf install rpm-build
```

### All platforms

1. Get the latest **stable** Rust toolchain via [rustup.rs](https://rustup.rs/).

1. Get the latest version 8 LTS release of Node.js (>=8.10) and the latest version of
   yarn (>=1.7.0).
   #### macOS
   ```bash
   brew install node@8 yarn
   export PATH="/usr/local/opt/node@8/bin:$PATH"
   ```

   #### Linux
   Just download and unpack the `node-v8.xxxx.tar.xz` tarball and add its `bin` directory to your
   `PATH`. Then install yarn with the help of the instructions on their website.

   #### Windows
   Download the Node.js and yarn installers from their official websites.

## Building and packaging the app

The simplest way to build the entire app and generate an installer is to just run the build script.
`--dev-build` is added to skip some release checks and signing of the binaries:
```bash
./build.sh --dev-build
```
This should produce an installer exe, pkg or rpm+deb file in the `dist/` directory.

Building this requires at least 1GB of memory.

If you want to build each component individually, or run in development mode, read the following
sections.

## Building and running mullvad-daemon

1. Firstly, one should source `env.sh` to set the default environment variables.
   One can also source the variables on Powershell with `env.ps1`,
   however most of our scripts require bash.
   ```bash
   source env.sh
   # Or if you use Powershell:
   . .\env.ps1
   ```

1. If you are on Windows, then you have to build the C++ libraries before compiling the daemon:
    ```bash
    bash ./build_windows_modules.sh --dev-build
    ```

1. Build the system daemon plus the other Rust tools and programs:
    ```
    cargo build
    ```

1. Copy the OpenVPN binary, and our plugin for it, to the directory we will use as resource
   directory. If you want to use any other directory, you would need to copy even more files.
   ```bash
   cp dist-assets/binaries/<platform>/openvpn[.exe] dist-assets/
   cp target/debug/*talpid_openvpn_plugin* dist-assets/
   ```


1. Run the daemon with verbose logging with:
    ```
    sudo MULLVAD_RESOURCE_DIR="./dist-assets" ./target/debug/mullvad-daemon -vv
    ```
    It must run as root since it modifies the firewall and sets up virtual network interfaces
    etc.

### Environment variables controlling the execution

* `TALPID_FIREWALL_DEBUG` - Helps debugging the firewall. Does different things depending on
  platform:
  * Linux: Set to `"1"` to add packet counters to all firewall rules.
  * macOS: Makes rules log the packets they match to the `pflog0` interface.
    * Set to `"all"` to add logging to all rules.
    * Set to `"pass"` to add logging to rules allowing packets.
    * Set to `"drop"` to add logging to rules blocking packets.

* `TALPID_DNS_MODULE` - Allows changing the method that will be used for DNS configuration on Linux.
  By default this is automatically detected, but you can set it to one of the options below to
  choose a specific method:
    * `"static-file"`: change the `/etc/resolv.conf` file directly
    * `"resolvconf"`: use the `resolvconf` program
    * `"systemd"`: use systemd's `resolved` service through DBus
    * `"network-manager"`: use `NetworkManager` service through DBus


## Building and running the Electron GUI app

1. Go to the `gui` directory
   ```bash
   cd gui
   ```

1. Install all the JavaScript dependencies by running:
    ```bash
    yarn install
    ```

1. Start the GUI in development mode by running:
    ```bash
    yarn develop
    ```

If you change any javascript file while the development mode is running it will automatically
transpile and reload the file so that the changes are visible almost immediately.

Please note that the GUI needs a running daemon to connect to in order to work. See
[Building and running mullvad-daemon](#building-and-running-mullvad-daemon) for instruction on how
to do that before starting the GUI.

The GUI will need to resolve the path to the `problem-report` tool. In development mode this
defaults to `<repo>/target/debug/`, but can be configured with the `MULLVAD_PATH` environment
variable.


## Making a release

When making a real release there are a couple of steps to follow. `<VERSION>` here will denote
the version of the app you are going to release. For example `2018.3-beta1` or `2018.4`.

1. Follow the [Install toolchains and dependencies](#install-toolchains-and-dependencies) steps
   if you have not already completed them.

1. Make sure the `CHANGELOG.md` is up to date and has all the changes present in this release.
   Also change the `[Unreleased]` header into `[<VERSION>] - <DATE>` and add a new `[Unreleased]`
   header at the top. Push this, get it reviewed and merged.

1. Run `./prepare_release.sh <VERSION>`. This will do the following for you:
    1. Check if your repository is in a sane state and the given version has the correct format
    1. Update `package.json` with the new version and commit that
    1. Add a signed tag to the current commit with the release version in it

    Please verify that the script did the right thing before you push the commit and tag it created.

1. When building for macOS, the following environment variables must be set:
   * `CSC_LINK` - The path to the `.p12` certificate file with the Apple application signing keys.
     This file must contain both the "Developer ID Application" and the "Developer ID Installer"
     certificates + private keys. If this environment variable is missing `build.sh` will skip
     signing.
   * `CSC_KEY_PASSWORD` - The password to the file given in `CSC_LINK`. If this is not set then
     `build.sh` will prompt you for it. If you set it yourself, make sure to define it in such a
     way that it's not stored in your bash history:
     ```bash
     export HISTCONTROL=ignorespace
     export CSC_KEY_PASSWORD='my secret'
     ```

1. Run `./build.sh` on each computer/platform where you want to create a release artifact. This will
    do the following for you:
    1. Update `relays.json` with the latest relays
    1. Compile and package the app into a distributable artifact for your platform.

    Please pay attention to the output at the end of the script and make sure the version it says
    it built matches what you want to release.

## Running Integration Tests

The integration tests are located in the `mullvad-tests` crate. It uses a mock OpenVPN binary to
test the `mullvad-daemon`. To run the tests, the `mullvad-daemon` binary must be built first.
Afterwards, the tests should be executed with the `integration-tests` feature enabled. To simplify
this procedure, the `integration-tests.sh` script can be used to run all integration tests.


## Command line tools for Electron GUI app development

- `$ yarn develop` - develop app with live-reload enabled
- `$ yarn lint` - lint code
- `$ yarn pack:<OS>` - prepare app for distribution for your platform. Where `<OS>` can be
  `linux`, `mac` or `win`
- `$ yarn test` - run tests

## Repository structure

### Electron GUI app and electron-builder packaging assets
- **gui/packages/**
  - **components/** - Platform agnostic shared react components
  - **desktop/** - The desktop implementation
    - **assets/** - graphical assets and stylesheets
    - **src/**
      - **main/**
        - **index.ts** - entry file for the main process
      - **renderer/**
        - **app.ts** - entry file for the renderer process
        - **routes.ts** - routes configurator
        - **transitions.ts** - transition rules between views
      - **config.json** - App color definitions and URLs to external resources
    - **test/** - Electron GUI tests
- **dist-assets/** - Icons, binaries and other files used when creating the distributables
  - **binaries/** - Git submodule containing binaries bundled with the app. For example the
    statically linked OpenVPN binary. See the README in the submodule for details
  - **linux/** - Scripts and configuration files for the deb and rpm artifacts
  - **pkg-scripts/** - Scripts bundled with and executed by the macOS pkg installer
  - **windows/** - Windows NSIS installer configuration and assets
  - **api_root_ca.pem** - The root CA for the api.mullvad.net endpoint. The app uses certificate
    pinning
  - **ca.crt** - The Mullvad relay server root CA. Bundled with the app and only OpenVPN relays
    signed by this CA are trusted


### Building, testing and misc
- **build_windows_modules.sh** - Compiles the C++ libraries needed on Windows
- **build.sh** - Sanity checks the working directory state and then builds release artifacts for
  the app

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
| Windows | `%LOCALAPPDATA%\Mullvad VPN\`

#### Logs

The log directory can be changed by setting the `MULLVAD_LOG_DIR` environment variable.

| Platform | Path |
|----------|------|
| Linux | `/var/log/mullvad-vpn/` + systemd |
| macOS | `/var/log/mullvad-vpn/` |
| Windows | `C:\ProgramData\Mullvad VPN\` |

#### Cache

The cache directory can be changed by setting the `MULLVAD_CACHE_DIR` environment variable.

| Platform | Path |
|----------|------|
| Linux | `/var/cache/mullvad-vpn/` |
| macOS | `/var/root/Library/Caches/mullvad-vpn/` |
| Windows | `%LOCALAPPDATA%\Mullvad VPN\` |

#### RPC address file

The full path to the RPC address file can be changed by setting the `MULLVAD_RPC_SOCKET_PATH`
environment variable.

| Platform | Path |
|----------|------|
| Linux | `/var/run/mullvad-vpn` |
| macOS | `/var/run/mullvad-vpn` |
| Windows | `//./pipe/Mullvad VPN` |

### GUI

The GUI has a specific settings file that is configured for each user. The path is set in the
`gui/packages/desktop/main/gui-settings.ts` file.

| Platform | Path |
|----------|------|
| Linux | `$XDG_CONFIG_HOME/Mullvad VPN/gui_settings.json` |
| macOS | `~/Library/Application Support/Mullvad VPN/gui_settings.json` |
| Windows | `%LOCALAPPDATA%\Mullvad VPN\gui_settings.json` |

## Audits, pentests and external security reviews

Mullvad has used external pentesting companies to carry out security audits of this VPN app. Read
more about them in the [audits readme](./audits/README.md)

## Quirks

- If you want to modify babel-configurations please note that `BABEL_ENV=development` must be used
  for [react-native](https://github.com/facebook/react-native/issues/8723)

# License

Copyright (C) 2017  Amagicom AB

This program is free software: you can redistribute it and/or modify it under the terms of the
GNU General Public License as published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.

For the full license agreement, see the LICENSE.md file


[Mullvad's Open Source page]: https://mullvad.net/en/guides/open-source/
