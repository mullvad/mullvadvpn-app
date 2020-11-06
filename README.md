# Mullvad VPN desktop and mobile app

Welcome to the Mullvad VPN client app. This repository contains all the source code for the
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
| Windows     | 7, 8.1 and 10      |
| macOS       | The three latest major releases |
| Linux (Ubuntu)| The two newest LTS releases and the two newest non-LTS releases |
| Linux (Fedora) | The versions that are not yet [EOL](https://fedoraproject.org/wiki/End_of_life) |
| Linux (Debian) | The versions that are not yet [EOL](https://wiki.debian.org/DebianReleases) |
| Android | The four latest major releases|
| iOS         | 13 and newer       |

On Linux we test using the Gnome desktop environment. The app should, and probably does work
in other DEs, but we don't regularly test those.

## Features

Here is a table containing the features of the app across platforms. This reflects the current
state of latest master, not necessarily any existing release.

|                               | Windows | Linux | macOS | Android |
|-------------------------------|:-------:|:-----:|:-----:|:-------:|
| OpenVPN                       |    ✓    |   ✓   |   ✓   |         |
| WireGuard                     |    ✓    |   ✓   |   ✓   |    ✓    |
| OpenVPN over Shadowsocks      |    ✓    |   ✓   |   ✓   |         |
| Optional local network access |    ✓    |   ✓   |   ✓   |    ✓    |

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
build scripts for third party code we need to bundle with the app. Such as OpenVPN, Shadowsocks
etc.

This submodule conforms to the same integrity/security standards as this repository. Every merge
commit should be signed. And this main repository should only ever point to a signed merge commit
of the binaries submodule.

See the [binaries submodule's](https://github.com/mullvad/mullvadvpn-app-binaries) README for more
details about that repository.

## Install toolchains and dependencies

Follow the instructions for your platform, and then the [All platforms](#all-platforms)
instructions.

These instructions are probably not complete. If you find something more that needs installing
on your platform please submit an issue or a pull request.

### Windows

The host has to have the following installed:

- Microsoft's _Build Tools for Visual Studio 2019_ (a regular installation of Visual Studio 2019
  Community edition works as well).

- Windows 10 SDK.

- `msbuild.exe` available in `%PATH%`. If you installed Visual Studio Community edition, the
  binary can be found under:
  ```
  C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\MSBuild\Current\Bin\amd64
  ```

- `bash` installed as well as a few base unix utilities, including `sed` and `tail`.
  The environment coming with [Git for Windows] works fine.
- `gcc` for CGo.

[Git for Windows]: https://git-scm.com/download/win

### Linux

#### Debian/Ubuntu
```bash
# For building the daemon
sudo apt install gcc libdbus-1-dev
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

### Android

These instructions are for building the app for Android **under Linux**.

#### Download and install the JDK
```bash
sudo apt install zip default-jdk
```

#### Download and install the SDK

The SDK should be placed in a separate directory, like for example `~/android` or `/opt/android`.
This directory should be exported as the `$ANDROID_HOME` environment variable.

```bash
cd /opt/android     # Or some other directory to place the Android SDK
export ANDROID_HOME=$PWD

wget https://dl.google.com/android/repository/commandlinetools-linux-6609375_latest.zip
unzip commandlinetools-linux-6609375_latest.zip
./tools/bin/sdkmanager "platforms;android-29" "build-tools;29.0.3" "platform-tools"
```

If `sdkmanager` fails to find the SDK root path, pass the option `--sdk_root=$ANDROID_HOME`
to the command above.

#### Download and install the NDK

The NDK should be placed in a separate directory, which can be inside the `$ANDROID_HOME` or in a
completely separate path. The extracted directory must be exported as the `$ANDROID_NDK_HOME`
environment variable.

```bash
cd "$ANDROID_HOME"  # Or some other directory to place the Android NDK
wget https://dl.google.com/android/repository/android-ndk-r20b-linux-x86_64.zip
unzip android-ndk-r20b-linux-x86_64.zip

cd android-ndk-r20b
export ANDROID_NDK_HOME="$PWD"
```

#### Docker

Docker is required to build `wireguard-go` for Android. Follow the [installation
instructions](https://docs.docker.com/engine/install/debian/) for your distribution.

#### Configuring Rust

These steps has to be done **after** you have installed Rust in the section below:

##### Install the Rust Android target

Some environment variables must be exported so that some Rust dependencies can be
cross-compiled correctly:
```
export NDK_TOOLCHAIN_DIR="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
export AR_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/aarch64-linux-android-ar"
export AR_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/arm-linux-androideabi-ar"
export AR_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/x86_64-linux-android-ar"
export AR_i686_linux_android="$NDK_TOOLCHAIN_DIR/i686-linux-android-ar"
export CC_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/aarch64-linux-android21-clang"
export CC_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi21-clang"
export CC_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/x86_64-linux-android21-clang"
export CC_i686_linux_android="$NDK_TOOLCHAIN_DIR/i686-linux-android21-clang"
```

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

##### Set up cargo to use the correct linker and archiver

This block assumes you installed everything under `/opt/android`, but you can install it wherever
you want as long as the `ANDROID_HOME` variable is set accordingly.

Add to `~/.cargo/config.toml`:
```
[target.aarch64-linux-android]
ar = "/opt/android/android-ndk-r20b/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar"
linker = "/opt/android/android-ndk-r20b/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"

[target.armv7-linux-androideabi]
ar = "/opt/android/android-ndk-r20b/toolchains/llvm/prebuilt/linux-x86_64/bin/arm-linux-androideabi-ar"
linker = "/opt/android/android-ndk-r20b/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi21-clang"

[target.x86_64-linux-android]
ar = "/opt/android/android-ndk-r20b/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android-ar"
linker = "/opt/android/android-ndk-r20b/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang"

[target.i686-linux-android]
ar = "/opt/android/android-ndk-r20b/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android-ar"
linker = "/opt/android/android-ndk-r20b/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android21-clang"
```

#### Signing key for release APKs (optional)

In order to build release APKs, they need to be signed. First, a signing key must be generated and
stored in a keystore file. In the example below, the keystore file will be
`/home/user/app-keys.jks` and will contain a key called `release`.

```
keytool -genkey -v -keystore /home/user/app-keys.jks -alias release -keyalg RSA -keysize 4096 -validity 10000
```

Fill in the requested information to generate the key and the keystore file. Suppose the file was
protected by a password `keystore-password` and the key with a password `key-password`. This
information should then be added to the `android/keystore.properties` file:

```
keyAlias = release
keyPassword = key-password
storeFile = /home/user/app-keys.jks
storePassword = keystore-password
```

### All platforms

1. Get the latest **stable** Rust toolchain via [rustup.rs](https://rustup.rs/).

1. *This can be skipped for Android builds*.

   Get the latest version 12 release of Node.js and the latest version of npm.
   #### macOS
   ```bash
   brew install node
   ```

   #### Linux
   Just download and unpack the `node-v12.xxxx.tar.xz` tarball and add its `bin` directory to your
   `PATH`.

   #### Windows
   Download the Node.js installer from the official website.

1. Install Go (ideally version `1.13.6`) by following the [official
   instructions](https://golang.org/doc/install).  Newer versions of Go may be used. Earlier
   versions may be used, but versions older than `1.12` are known to not work, newer versions may
   too. Since `cgo` is being used, make sure to have a C compiler in your path. [*On
   Windows*](https://github.com/golang/go/wiki/cgo#windows) `mingw`'s `gcc` compiler should work.
   `gcc` on most Linux distributions should work, and `clang` for MacOS.


## Building and packaging the app

### Desktop

The simplest way to build the entire app and generate an installer is to just run the build script.
`--dev-build` is added to skip some release checks and signing of the binaries:
```bash
./build.sh --dev-build
```
This should produce an installer exe, pkg or rpm+deb file in the `dist/` directory.

Building this requires at least 1GB of memory.

If you want to build each component individually, or run in development mode, read the following
sections.

### Android

Running the `build-apk.sh` script will build the necessary Rust daemon for all supported ABIs and
build the final APK:
```bash
./build-apk.sh
```

You may pass a `--dev-build` to build the Rust daemon and the UI in debug mode and sign the APK with
automatically generated debug keys:
```bash
./build-apk.sh --dev-build
```

If the above fails with an error related to compression, try allowing more memory to the JVM:
```bash
echo "org.gradle.jvmargs=-Xmx4608M" >> ~/.gradle/gradle.properties
./android/gradlew --stop
```

## Building and running mullvad-daemon on desktop

1. Firstly, on MacOS and Linux, one should source `env.sh` to set the default environment variables.
   ```bash
   source env.sh
   ```

1. If you are on Windows, then you have to build the C++ libraries before compiling the daemon:
    ```bash
    bash ./build_windows_modules.sh --dev-build
    ```

1. Build the system daemon plus the other Rust tools and programs:
    ```
    cargo build
    ```

1. Copy the OpenVPN and Shadowsocks binaries, and our plugin for it, to the directory we will
   use as resource directory. If you want to use any other directory, you would need to copy
   even more files.
   ```bash
   cp dist-assets/binaries/<platform>/{openvpn, sslocal}[.exe] dist-assets/
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

* `TALPID_FORCE_USERSPACE_WIREGUARD` - Forces the daemon to use the userspace implementation of
   WireGuard on Linux.

* `TALPID_FORCE_NM_WIREGUARD` - Forces the daemon to use NetworkManager to create a WireGuard
  device instead of relying on netlink. This is highly inadvisable currently, as NetworkManager is
  exhibiting buggy behavior.


## Building and running the desktop Electron GUI app

1. Go to the `gui` directory
   ```bash
   cd gui
   ```

1. Install all the JavaScript dependencies by running:
    ```bash
    npm install
    ```

1. Start the GUI in development mode by running:
    ```bash
    npm run develop
    ```

If you change any javascript file while the development mode is running it will automatically
transpile and reload the file so that the changes are visible almost immediately.

Please note that the GUI needs a running daemon to connect to in order to work. See
[Building and running mullvad-daemon](#building-and-running-mullvad-daemon) for instruction on how
to do that before starting the GUI.

### Supported environment variables

1. `MULLVAD_PATH` - Allows changing the path to the folder with the `mullvad-problem-report` tool
    when running in development mode. Defaults to: `<repo>/target/debug/`.


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

1. When building for Windows or macOS, the following environment variables must be set:
   * `CSC_LINK` - The path to the certificate used for code signing.
      * Windows: A `.pfx` certificate.
      * macOS: A `.p12` certificate file with the Apple application signing keys.
        This file must contain both the "Developer ID Application" and the "Developer ID Installer"
        certificates + private keys.
   * `CSC_KEY_PASSWORD` - The password to the file given in `CSC_LINK`. If this is not set then
      `build.sh` will prompt you for it. If you set it yourself, make sure to define it in such a
      way that it's not stored in your bash history:
      ```bash
      export HISTCONTROL=ignorespace
      export CSC_KEY_PASSWORD='my secret'
      ```

   * *macOS only*:
      * `NOTARIZE_APPLE_ID` - The AppleId to use when notarizing the app. Only needed on release builds
      * `NOTARIZE_APPLE_ID_PASSWORD` - The AppleId password for the account in `NOTARIZE_APPLE_ID`.
         Don't use the real AppleId password! Instead create an app specific password and add that to
         your keyring. See this documentation: https://github.com/electron/electron-notarize#safety-when-using-appleidpassword

         Summary:
         1. Generate app specific password on Apple's AppleId management portal.
         2. Run `security add-generic-password -a "<apple_id>" -w <app_specific_password> -s "something_something"`
         3. Set `NOTARIZE_APPLE_ID_PASSWORD="@keychain:something_something"`.

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
| macOS | `/var/root/Library/Caches/mullvad-vpn/` |
| Windows | `%LOCALAPPDATA%\Mullvad VPN\` |
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

Icons such as the logo and menubar icons are automatically generated. The source files are:
| Path | Usage |
|------|-------|
| `graphics/icon.svg` | The logo icon used for e.g. application icon and in app logo |
| `graphics/icon-mono.svg` | The logo icon used for the android notification icon |
| `graphics/icon-square.svg` | Logo icon used to generate the iOS application icon |
| `gui/assets/images/*.svg` | Icons used to generate iOS icons and used in the desktop app |
| `gui/assets/images/menubar icons/svg/*.svg` | The frames for the menubar icon |

Generate desktop icon by running
```bash
gui/scripts/build-logo-icons.sh
```

Generate android icons
```bash
android/generate-pngs.sh
```

Generate iOS icon and assets
```bash
ios/convert-assets.rb --app-icon
ios/convert-assets.rb --import-desktop-assets
ios/convert-assets.rb --additional-assets
```

Generate desktop menubar icons
```bash
gui/scripts/build-menubar-icons.sh
```

The menubar icons are described futher [here](./gui/assets/images/menubar%20icons/README.md).

## Locales and translations

Instructions for how to handle locales and translations are found
[here](./gui/locales/README.md).

For instructions specific to the Android app, see [here](./android/README.md).

## Audits, pentests and external security reviews

Mullvad has used external pentesting companies to carry out security audits of this VPN app. Read
more about them in the [audits readme](./audits/README.md).

# License

Copyright (C) 2020  Mullvad VPN AB

This program is free software: you can redistribute it and/or modify it under the terms of the
GNU General Public License as published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.

For the full license agreement, see the LICENSE.md file


[Mullvad's Open Source page]: https://mullvad.net/en/guides/open-source/
