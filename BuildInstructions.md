These are instructions on how to build the app on desktop platforms. See the
[readme](./README.md#building-the-app) for help build building on other platforms.

# Install toolchains and dependencies

These instructions are probably not complete. If you find something more that needs installing
on your platform please submit an issue or a pull request.

## All platforms

- Get the latest **stable** Rust toolchain via [rustup.rs](https://rustup.rs/).

- You need Node.js and npm. You can find the exact versions in the `volta` section of
  `gui/package.json`. The toolchain is managed by volta.

  - Linux & macOS

    ```bash
    cargo install --git https://github.com/volta-cli/volta && volta setup
    ````

  - Windows

    Install the `msi` hosted here: https://github.com/volta-cli/volta

- Install Go (ideally version `1.18`) by following the [official
  instructions](https://golang.org/doc/install).  Newer versions may work
  too.

- Install a protobuf compiler (version 3 and up), it can be installed on most major Linux distros
  via the package name `protobuf-compiler`, `protobuf` on macOS via Homebrew, and on Windows
  binaries are available on their GitHub [page](https://github.com/protocolbuffers/protobuf/releases)
  and they have to be put in `%PATH`. An additional package might also be required depending on
  Linux distro:
  - `protobuf-devel` on Fedora.
  - `libprotobuf-dev` on Debian/Ubuntu.

## Linux

### Debian/Ubuntu

```bash
# For building the daemon
sudo apt install gcc libdbus-1-dev
# For building the installer
sudo apt install rpm
```

### Fedora/RHEL

```bash
# For building the daemon
sudo dnf install dbus-devel
# For building the installer
sudo dnf install rpm-build
```

### Cross-compiling for ARM64

By default, the app will build for the host platform. It is also possible to cross-compile the app
for ARM64 on x64.

#### Debian

```bash
# As root
dpkg --add-architecture arm64 && \
    apt update && \
    apt install libdbus-1-dev:arm64 gcc-aarch64-linux-gnu
```

```bash
rustup target add aarch64-unknown-linux-gnu
```

To make sure the right linker and libraries are used, add the following to `~/.cargo/config.toml`:

```
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.aarch64-unknown-linux-gnu.dbus]
rustc-link-search = ["/usr/aarch64-linux-gnu/lib"]
rustc-link-lib = ["dbus-1"]
```

## Windows

The host has to have the following installed:

- Microsoft's _Build Tools for Visual Studio 2022_ (a regular installation of Visual Studio 2022
  Community or Pro edition works as well).

- Windows 10 (or Windows 11) SDK.

- `msbuild.exe` available in `%PATH%`. If you installed Visual Studio Community edition, the
  binary can be found under:
  ```
  C:\Program Files\Microsoft Visual Studio\2022\Community\MSBuild\Current\Bin\amd64
  ```

- `bash` installed as well as a few base unix utilities, including `sed` and `tail`.
  You are recommended to use [Git for Windows].

- `mingw`'s `gcc` compiler is required for CGo.

[Git for Windows]: https://git-scm.com/download/win

## macOS

The host has to have the following installed:

- A recent version of `bash`. The default version in macOS (3.2.57) isn't supported.

- `clang` is required for CGo.

# Building and packaging the app

The simplest way to build the entire app and generate an installer is to just run the build script.
`--optimize` can be added to enable compiler optimizations. This will take longer to build but will
produce a smaller installer and installed binaries:
```bash
./build.sh [--optimize]
```
This should produce an installer exe, pkg or rpm+deb file in the `dist/` directory.

Building this requires at least 1GB of memory.

## Notes on targeting ARM64

### macOS

By default, `build.sh` produces a pkg for your current architecture only. To build a universal
app that works on both Intel and Apple Silicon macs, build with `--universal`.

### Linux

To cross-compile for ARM64 rather than the current architecture, set the `TARGETS` environment
variable to `aarch64-unknown-linux-gnu`:

```bash
TARGETS="aarch64-unknown-linux-gnu" ./build.sh
```

## Notes on building on ARM64 Linux hosts

Due to inability to build the management interface proto files on ARM64 (see
[this](https://github.com/grpc/grpc-node/issues/1497) issue), building on ARM64 must be done in
2 stages:

1. Build management interface proto files on another platform than arm64 Linux
2. Use the built proto files during the main build by setting the
   `MANAGEMENT_INTERFACE_PROTO_BUILD_DIR` environment variable to the path the proto files

To build the management interface proto files there is a script (execute it on another platform than
ARM64 Linux):

```bash
cd gui/scripts
npm ci
./build-proto.sh
```

After that copy the files from `gui/src/main/management_interface/` and
`gui/build/src/main/management_interface/` directories into a single directory, and set the value
of `MANAGEMENT_INTERFACE_PROTO_BUILD_DIR` to that directory while running the main build.

When all is done run the main build. Assuming that you copied the proto files into `/tmp/management_interface_proto`
directory, the build command will look as follows:

```bash
MANAGEMENT_INTERFACE_PROTO_BUILD_DIR=/tmp/management_interface_proto ./build.sh --dev-build
```

On Linux, you may also have to specify `USE_SYSTEM_FPM=true` to generate the deb/rpm packages.

# Building and running mullvad-daemon

This section is for building the system service individually.

1. Source `env.sh` to set the default environment variables:
    ```bash
    source env.sh
    ```

1. On Windows, build the C++ libraries:
    ```bash
    ./build-windows-modules.sh
    ```

1. Build the system daemon plus the other Rust tools and programs:
    ```bash
    cargo build
    ```

1. Copy the OpenVPN binaries, and our plugin for it, to the directory we will
    use as resource directory. If you want to use any other directory, you would need to copy
    even more files.
    ```bash
    cp dist-assets/binaries/<platform>/openvpn[.exe] dist-assets/
    cp target/debug/*talpid_openvpn_plugin* dist-assets/
    cp dist-assets/binaries/x86_64-pc-windows-msvc/wintun.dll target/debug/
    ```

1. On Windows, the daemon must be run as the SYSTEM user. You can use
    [PsExec](https://docs.microsoft.com/en-us/sysinternals/downloads/psexec) to launch
    an elevated bash instance before starting the daemon in it:
    ```
    psexec64 -i -s bash.exe
    ```

1. Run the daemon with verbose logging (from the root directory of the project):
    ```bash
    sudo MULLVAD_RESOURCE_DIR="./dist-assets" ./target/debug/mullvad-daemon -vv
    ```
    Leave out `sudo` on Windows. The daemon must run as root since it modifies the firewall and sets
    up virtual network interfaces etc.

# Building and running the GUI app

This section is for building the GUI app individually.

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
[Building and running mullvad-daemon](#building-and-running-mullvad-daemon) for instructions
on how to do that before starting the GUI.
