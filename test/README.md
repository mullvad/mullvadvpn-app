# Project structure

## test-manager

The client part of the testing environment. This program runs on the host and connects over a
virtual serial port to the `test-runner`.

The tests themselves are defined in this package, using the interface provided by `test-runner`.

## test-runner

The server part of the testing environment. This program runs in guest VMs and provides the
`test-manager` with the building blocks (RPCs) needed to create tests.

## test-rpc

A support library for the other two packages. Defines an RPC interface, transports, shared types,
etc.

# Prerequisities

For macOS, the host machine must be macOS. All other platforms assume that the host is Linux.

## All platforms

* Get the latest stable Rust from https://rustup.rs/.

## macOS

Normally, you would use Tart here. It can be installed with Homebrew. You'll also need
`wireguard-tools`, a protobuf compiler, and OpenSSL:

```bash
brew install cirruslabs/cli/tart wireguard-tools pkg-config openssl protobuf
```

### Wireshark

Wireshark is also required. More specifically, you'll need `wireshark-chmodbpf`, which can be found
in the Wireshark installer here: https://www.wireshark.org/download.html

You also need to add the current user to the `access_bpf` group:

```bash
dseditgroup -o edit -a THISUSER -t user access_bpf
```

This lets us monitor traffic on network interfaces without root access.

## Linux

For running tests on Linux and Windows guests, you will need these tools and libraries:

```bash
dnf install git gcc protobuf-devel libpcap-devel qemu \
    podman golang-github-rootless-containers-rootlesskit slirp4netns dnsmasq \
    dbus-devel pkgconf-pkg-config swtpm edk2-ovmf \
    wireguard-tools
```

# Building the test runner

Building the `test-runner` binary is done with the `build-runner.sh` script.
Currently, only `x86_64` platforms are supported for Windows/Linux and `ARM64` (Apple Silicon) for macOS.

For example, building `test-runner` for Windows would look like this:

``` bash
./scripts/container-run.sh ./scripts/build-runner.sh windows
```

## Linux
Using `podman` is the recommended way to build the `test-runner`. See the [Linux section under Prerequisities](#Prerequisities) for more details.

``` bash
./scripts/container-run.sh ./scripts/build-runner.sh linux
```

## macOS

``` bash
./scripts/build-runner.sh macos
```

## Windows
The `test-runner` binary for Windows may be cross-compiled from a Linux host.

``` bash
./scripts/container-run.sh ./scripts/build-runner.sh windows
```

# Building base images

See [`BUILD_OS_IMAGE.md`](./docs/BUILD_OS_IMAGE.md) for how to build images for running tests on.

# Running tests

See `cargo run --bin test-manager` for details.

## Linux

Here is an example of how to create a new OS configuration and then run all tests:

```bash
# Create or edit configuration
# The image is assumed to contain a test runner service set up as described in ./docs/BUILD_OS_IMAGE.md
cargo run --bin test-manager set debian11 qemu ./os-images/debian11.qcow2 linux \
    --package-type deb --architecture x64 \
    --provisioner ssh --ssh-user test --ssh-password test

# Try it out to see if it works - you should reach the VM's graphical desktop environment
cargo run --bin test-manager run-vm debian11

# Run all tests
cargo run --bin test-manager run-tests debian11 \
    --display \
    --account 0123456789 \
    --app-package <git hash or tag> \
    --app-package-to-upgrade-from 2023.2
```

## macOS

Here is an example of how to create a new OS configuration (on Apple Silicon) and then run all
tests:

```bash
# Download some VM image
tart clone ghcr.io/cirruslabs/macos-ventura-base:latest ventura-base

# Create or edit configuration
# Use SSH to deploy the test runner since the image doesn't contain a runner
cargo run --bin test-manager set macos-ventura tart ventura-base macos \
    --architecture aarch64 \
    --provisioner ssh --ssh-user admin --ssh-password admin

# Try it out to see if it works
#cargo run -p test-manager run-vm macos-ventura

# Run all tests
cargo run --bin test-manager run-tests macos-ventura \
    --display \
    --account 0123456789 \
    --app-package <git hash or tag> \
    --app-package-to-upgrade-from 2023.2
```

## Note on `scripts/ci-runtests.sh`

Account tokens are read (newline-delimited) from the path specified by the environment variable
`ACCOUNT_TOKENS`. Round robin is used to select an account for each VM.
