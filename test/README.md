# Mullvad VPN end to end test framework

## Project structure

### test-manager

The client part of the testing environment. This program runs on the host and connects over a
virtual serial port to the `test-runner`.

The tests themselves are defined in this package, using the interface provided by `test-runner`.

### test-runner

The server part of the testing environment. This program runs in guest VMs and provides the
`test-manager` with the building blocks (RPCs) needed to create tests.

### test-rpc

A support library for the other two packages. Defines an RPC interface, transports, shared types,
etc.


## Prerequisites

For macOS, the host machine must be macOS. All other platforms assume that the host is Linux.

### All platforms

* Get the latest stable Rust from https://rustup.rs/.

### macOS

Normally, you would use Tart here. It can be installed with Homebrew. You'll also need
`wireguard-tools`, a protobuf compiler, and OpenSSL:

```bash
brew install cirruslabs/cli/tart wireguard-tools pkg-config openssl protobuf
```

#### Wireshark

Wireshark is also required. More specifically, you'll need `wireshark-chmodbpf`, which can be found
in the Wireshark installer here: https://www.wireshark.org/download.html

You also need to add the current user to the `access_bpf` group:

```bash
dseditgroup -o edit -a THISUSER -t user access_bpf
```

This lets us monitor traffic on network interfaces without root access.

### Linux

For running tests on Linux and Windows guests, you will need these tools and libraries:

```bash
dnf install git gcc protobuf-devel libpcap-devel qemu \
    podman golang-github-rootless-containers-rootlesskit slirp4netns dnsmasq \
    dbus-devel pkgconf-pkg-config swtpm edk2-ovmf \
    wireguard-tools
```

## Setting up testing environment

First you need to build the images for running tests on, see [`BUILD_OS_IMAGE.md`](./docs/BUILD_OS_IMAGE.md). The `test-manager` then needs to be configured to use the image.

Here is an example of how to create a new OS configuration for Linux and macOS:

### Linux

```bash
# Create or edit configuration
# The image is assumed to contain a test runner service set up as described in ./docs/BUILD_OS_IMAGE.md
cargo run --bin test-manager set debian11 qemu ./os-images/debian11.qcow2 linux \
    --package-type deb --architecture x64 \
    --provisioner ssh --ssh-user test --ssh-password test

# Try it out to see if it works - you should reach the VM's graphical desktop environment
cargo run --bin test-manager run-vm debian11
```

### macOS


```bash
# Download some VM image
tart clone ghcr.io/cirruslabs/macos-ventura-base:latest ventura-base

# Create or edit configuration
# Use SSH to deploy the test runner since the image doesn't contain a runner
cargo run --bin test-manager set macos-ventura tart ventura-base macos \
    --architecture aarch64 \
    --provisioner ssh --ssh-user admin --ssh-password admin

# Try it out to see if it works
cargo run -p test-manager run-vm macos-ventura
```

## Testing the app

To automatically download and test a pre-built version of the app, use the `test-by-version.sh` script, see `test-by-version.sh --help` for instructions.

To manually invoke `test-manager`, start by checking out the desired git version of this repo. Next, [build the app](../BuildInstructions.md) for the target platform then build the GUI test binary using `$(cd ../gui && npm run build-test-executable)`. The newly built packages will be located in the `../dist` folder by default.

Next: build the `test-runner`

### Building the test runner

Building the `test-runner` binary is done with the `build/test-runner.sh` script.
Currently, only `x86_64` platforms are supported for Windows/Linux and `ARM64` (Apple Silicon) for macOS.

For example, building `test-runner` for Windows would look like this:

``` bash
./scripts/container-run.sh ./scripts/build/test-runner.sh windows
```

#### Linux
Using `podman` is the recommended way to build the `test-runner`. See the [Linux section under Prerequisities](#prerequisites) for more details.

``` bash
./scripts/container-run.sh ./scripts/build/test-runner.sh linux
```

#### macOS

``` bash
./scripts/build/test-runner.sh macos
```

#### Windows
The `test-runner` binary for Windows may be cross-compiled from a Linux host.

``` bash
./scripts/container-run.sh ./scripts/build/test-runner.sh windows
```

### Running the tests

After configuring the VM image using `test-manager set` and building the required packages (see [previous step](#setting-up-testing-environment)), `test-manager run-tests` is used to launch the tests. See `cargo run --bin test-manager -- run-tests --help` for details.

Here is an example of how to run all tests using the Linux/macOS VM we set up earlier:

#### Linux

```bash
# Run all tests
cargo run --bin test-manager run-tests --vm debian11 \
    --display \
    --account 0123456789 \
    --app-package <git hash or tag> \
    --app-package-to-upgrade-from 2023.2
```

#### macOS

```bash
# Run all tests
cargo run --bin test-manager run-tests --vm macos-ventura \
    --display \
    --account 0123456789 \
    --app-package <git hash or tag> \
    --app-package-to-upgrade-from 2023.2
```

## Note on `scripts/run/ci.sh`

`scripts/run/ci.sh` is the script that GitHub actions uses to invokes the `test-manager`, with similar functionality as `test-by-version.sh`. Note that account numbers are read (newline-delimited) from the path specified by the environment variable `ACCOUNT_TOKENS`. Round robin is used to select an account for each VM.
