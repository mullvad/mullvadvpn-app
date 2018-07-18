# Mullvad VPN desktop and mobile app

The system service/daemon, GUI and CLI for the Mullvad VPN app.

## Status

There are built and signed releases for macOS available on
[our website](https://mullvad.net/download/) and on
[github](https://github.com/mullvad/mullvadvpn-app/releases/).
Support for Linux, Windows, Android and iOS is in the making.

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

For Debian based distributions, you need to install the following. For other distributions you need
the equivalent packages:
```bash
sudo apt install gcc libssl-dev libappindactor1
```

### All platforms

1. Get the latest stable Rust toolchain. This is easy with rustup, follow the instructions on
[rustup.rs](https://rustup.rs/).

1. Get Node.js (version 8 or 9) and the latest version of yarn. On macOS these can be installed via
homebrew:
    ```bash
    brew install node yarn
    ```

## Building and running mullvad-daemon

1. Firstly, one should source `env.sh` to set the default environment
   variables. One can also source the variables on Powershell with `env.ps1`,
   however most of our scripts require bash.

1. If you are on Windows, then you have to build the C++ libraries before compiling the daemon.
   Run `build_winfw.sh` to build a C++ library that sets firewall rules on Windows.
    ```bash
    bash build_winfw.sh
    ```

1. Build the daemon without optimizations (debug mode) with:
    ```
    cargo build
    ```

1. Run the daemon debug binary with verbose logging to the terminal with:
    ```
    sudo MULLVAD_RESOURCE_DIR="./dist-assets" ./target/debug/mullvad-daemon -vv
    ```
    It must run as root since it it modifies the firewall and sets up virtual network interfaces
    etc.

### Environment variables controlling the execution

* `TALPID_NFTABLES_COUNTERS` - Set to `"1"` to add packet counters to all firewall rules on
  Linux.


## Building and running the Electron GUI app

1. Install all the JavaScript dependencies by running:
    ```bash
    yarn install
    ```

1. Start the GUI in development mode by running:
    ```bash
    yarn workspace desktop develop
    ```

If you change any javascript file while the development mode is running it will automatically
transpile and reload the file so that the changes are visible almost immediately.

Please note that the GUI needs a running daemon to connect to in order to work. See
[Building and running mullvad-daemon](#building-and-running-mullvad-daemon) for instruction on how
to do that before starting the GUI.

The GUI will need to resolve the path to binaries. In development mode this defaults to
`./target/debug/`, but can be configured with the `MULLVAD_PATH` environment variable.


## Packaging the app

1. Follow the [Install toolchains and dependencies](#install-toolchains-and-dependencies) steps

1. Build the daemon in optimized release mode with:
    ```
    cargo build --release
    ```

1. Install all JavaScript dependencies (unless you already have) and package the application with:
    ```bash
    cd gui
    yarn install
    yarn pack:<OS>  # <OS> can be linux, mac or win
    ```
    This will create installation packages for windows, linux or macOS. Note that you have to have
    run `yarn install` at least once before this step to download the javascript dependencies.

    The artifact (.pkg, .deb, .msi) version is the `version` property of `package.json`.


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

- `$ yarn workspace desktop develop` - develop app with live-reload enabled
- `$ yarn flow` - type-check the code
- `$ yarn lint` - lint code
- `$ yarn pack:<OS>` - prepare app for distribution for your platform. Where `<OS>` can be
  `linux`, `mac` or `win`
- `$ yarn workspace desktop test` - run tests

## Repository structure

### Electron GUI and electron-builder packaging
- **app/**
  - **redux/** - state management
  - **components/** - components
  - **containers/** - containers that provide a glueing layer between components and redux
    actions/backend.
  - **lib/** - shared classes and utilities
  - **assets/** - graphical assets and stylesheets
  - **config.json** - links to external components
  - **app.js** - entry file for renderer process
  - **main.js** - entry file for background process
  - **routes.js** - routes configurator
  - **transitions.js** - transition rules between views
- **init.js** - entry file for electron, points to compiled **main.js**
- **scripts/** - support scripts for development
- **test/** - Electron GUI tests
- **dist-assets/** - Icons, binaries and other files used when creating the distributables
  - **binaries/** - Git submodule containing binaries bundled with the app. For example the
    statically linked OpenVPN binary. See the README in the submodule for details.
  - **pkg-scripts/** - Scripts bundled with and executed by the macOS pkg installer

### Building, testing and misc
- **build.sh** - Sanity checks the working directory state and then builds release artifacts for
  the app.
- **uninstall.sh** - Temporary script to help uninstall Mullvad VPN, all settings files, caches and
  logs.

### Daemon

The daemon is implemented in Rust and is implemented in several crates. The main, or top level,
crate that builds the final daemon binary is `mullvad-daemon` which then depend on the others.

In general one can look at the daemon as split into two parts, the crates starting with `talpid`
and the crates starting with `mullvad`. The `talpid` crates are supposed to be completely unrelated
to Mullvad specific things. A `talpid` crate is not allowed to know anything about the API through
which the daemon fetch Mullvad account details or download VPN server lists for example. The
`talpid` components should be viewed as a generic VPN client with extra privacy and anonymity
preserving features. The crates having `mullvad` in their name on the other hand make use of the
`talpid` components to build a secure and Mullvad specific VPN client.


- **Cargo.toml** - Main Rust workspace definition. See this file for which folders here are backend
  Rust crates.
- **mullvad-daemon/** - Main Rust crate building the daemon binary.


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

The full path to the RPC address file can be changed by setting the `MULLVAD_RPC_ADDRESS_PATH`
environment variable.

| Platform | Path |
|----------|------|
| Linux | `/tmp/.mullvad_rpc_address` |
| macOS | `/tmp/.mullvad_rpc_address` |
| Windows | `C:\ProgramData\Mullvad VPN\.mullvad_rpc_address` |

## Quirks

- If you want to modify babel-configurations please note that `BABEL_ENV=development` must be used
  for [react-native](https://github.com/facebook/react-native/issues/8723)

# License

Copyright (C) 2017  Amagicom AB

This program is free software: you can redistribute it and/or modify it under the terms of the
GNU General Public License as published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.

For the full license agreement, see the LICENSE.md file
