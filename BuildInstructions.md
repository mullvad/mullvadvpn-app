These are instructions on how to build the app on desktop platforms. See the
[readme](./README.md#building-the-app) for help building on other platforms.

# Install toolchains and dependencies

These instructions are probably not complete. If you find something more that needs installing
on your platform please submit an issue or a pull request.

## All platforms

- Get the latest **stable** Rust toolchain via [rustup.rs](https://rustup.rs/).
  - Install default targets and components needed for your platform:
    ```bash
    ./scripts/setup-rust android|ios|windows|linux|macos
     ```
  - (Optional) Run the following to install a git `post-checkout` hook that will automatically
    run the `setup-rust` script when the Rust version specified in the `rust-toolchain.toml` file changes:
    ```bash
    .scripts/setup-rust install-hook
    ```

- You need Node.js and npm. You can find the exact versions in the `volta` section of
  `desktop/package.json`. The toolchain is managed by volta.

  - Linux

    Follow instructions on [volta.sh](https://volta.sh/)

  - macOS

    ```bash
    brew install volta && volta setup
    ````

  - Windows

    Install the `msi` hosted here: https://github.com/volta-cli/volta

- Install Go (ideally version `1.21`) by following the [official instructions](https://golang.org/doc/install).
  Newer versions may work too.

- Install a protobuf compiler (version 3.15 and up), it can be installed on most major Linux distros
  via the package name `protobuf-compiler`, `protobuf` on macOS via Homebrew, and on Windows
  binaries are available on their GitHub [page](https://github.com/protocolbuffers/protobuf/releases)
  and they have to be put in `%PATH`. An additional package might also be required depending on
  Linux distro:
  - `protobuf-devel` on Fedora.
  - `libprotobuf-dev` on Debian/Ubuntu.

- **`bash` must be installed and available in PATH on all platforms**. This is required for building
  the desktop app:
- Bash version 4.0 or later is required for all platforms and must be added to your PATH environment variable.
  - Linux: Bash is typically installed by default, otherwise refer to your distribution for instructions on how to install it.
  - macOS: The default installed version (3.2.5) is not supported. Install a newer version via Homebrew: `brew install bash`
  - Windows: Install [Git for Windows] which includes Git Bash and other required unix utilities.

- Install `podman`: https://podman.io/


[Git for Windows]: https://git-scm.com/download/win

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

## Using nix devshell

This is supported on Linux (x86_64) as well as macOS (x86_64 and aarch64).
[Install nix](./android/docs/BuildInstructions.md#Build-using-nix-devshell) if you haven't already.

   ```bash
   nix develop
   ```

#### direnv

Provided in the repository root is a [direnv](https://direnv.net/) for automatically sourcing the devshell.
Allow it by executing `direnv allow .` (in `<repository>`) once.

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

- `bash` and base Unix utilities must be installed and available in PATH (see the requirement in
  the [All platforms](#all-platforms) section above).

- `zig` installed and available in `%PATH%`. 0.14 or later is recommended: https://ziglang.org/download/.

- `msbuild.exe` available in `%PATH%`. If you installed Visual Studio Community edition, the
  binary can be found under:

  ```
  C:\Program Files\Microsoft Visual Studio\2022\Community\MSBuild\Current\Bin\<arch>
  ```

  Where `<arch>` refers to the host architecture, either `amd64` or `arm64`.

  The environment can also be set up in bash by sourcing `vcvars.sh`: `. ./scripts/vcvars.sh`. Note
  that that script assumes that you're running VS 2022 Community.

- The `x86` target is required for building some NSIS plugins:

  ```bash
  rustup target add i686-pc-windows-msvc
  ```

### Cross-compiling for ARM64

By default, the app will build for the host platform. It is also possible to cross-compile the app
for ARM64 on x64. This requires:

- The ARM64 MSVC tools added to Visual Studio.

- `clang` (either directly from llvm.org or as part of Visual Studio) on the `PATH`.

- The `AArch64` target added to Rust:

```bash
rustup target add aarch64-pc-windows-msvc
```

### Compiling *on* Windows Arm

In addition to the above requirements:

- `clang` is required (can be found in the Visual Studio installer) in `PATH`.

  `INCLUDE` also needs to include the correct headers for clang. This can be found by running
  `vcvarsall.bat arm64` and typing `set INCLUDE`.

  The environment can also be set up in bash by sourcing `vcvars.sh`: `. ./scripts/vcvars.sh`. Note
  that that script assumes that you're running VS 2022 Community.

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

### Windows

To cross-compile for ARM64 from another host architecture, set the `TARGETS` environment
variable to `aarch64-pc-windows-msvc`:

```bash
TARGETS="aarch64-pc-windows-msvc" ./build.sh
```

## Notes on building on ARM64 Linux hosts

You may have to specify `USE_SYSTEM_FPM=true` to generate the deb/rpm packages.

# Building and running mullvad-daemon

This section is for building the system service individually.

1. On macOS, source `env.sh` to set the default environment variables:
    ```bash
    source env.sh
    ```

1. On Windows, build the C++ libraries:
    ```bash
    ./build-windows-modules.sh
    ```
    And copy the `wintun.dll` next to the daemon binary:
    ```bash
    cp dist-assets/binaries/x86_64-pc-windows-msvc/wintun.dll target/debug/
    ```

1. Build the system daemon plus the other Rust tools and programs:
    ```bash
    cargo build
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

# Building and running the desktop app

This section is for building the desktop app individually.

1. Go to the `desktop` directory
   ```bash
   cd desktop
   ```

1. Install all the JavaScript dependencies by running:
    ```bash
    npm install -w mullvad-vpn
    ```

1. Start the Electron app in development mode by running:
    ```bash
    npm run -w mullvad-vpn develop
    ```

If you change any javascript file while the development mode is running it will automatically
transpile and reload the file so that the changes are visible almost immediately.

Please note that the Electron app needs a running daemon to connect to in order to work. See
[Building and running mullvad-daemon](#building-and-running-mullvad-daemon) for instructions
on how to do that before starting the Electron app.
