# Build instructions

This document aims to explain how to build the Mullvad Android app. It's strongly recommended and
primarily supported to build the app using the provided container, as it ensures the correct build
environment.

## Build process

The build process consist of two main steps. First building the native libraries (`mullvad-daemon`
and `wireguard-go`) and then building the Android app/project which will bundle the previously built
native libraries. Building the native libraries requires some specific toolchains and packages to be
installed, so it's recommended to build using the provided build script and container image.

The native libraries doesn't have to be rebuilt very often, only when including daemon changes or
after cleaning the project, so apart from that it's possible to build the Android app/project using
the Gradle CLI or the Android Studio GUI.

## Build with provided container (recommended)

> __*NOTE:*__ Build with provided container is only supported on Linux and may not work on other
> platforms.

Building both the native libraries and Android project can easily be achieved by running the
[containerized-build.sh](../building/containerized-build.sh) script, which helps using the correct
tag and mounting volumes. The script relies on [podman](https://podman.io/getting-started/installation.html)
by default, however another container runner such as [docker](https://docs.docker.com/get-started/)
can be used by setting the `CONTAINER_RUNNER` environment variable.

After the native libraries have been built, subsequent builds can that doesn't rely on changes to
the native libraries can be ran using the Gradle CLI or the Android Studio GUI. This requires
either:
* Rust to be installed, since a tooled called `mullvad-version` is used to resolved the version
  information for the Android app.

or

* Specifying custom version information by following [these instructions](#override-version-code-and-version-name).

### Setup:

- Install [podman](https://podman.io/getting-started/installation.html) and make sure it's
  configured to run in rootless mode.

- OPTIONAL: Get the latest **stable** Rust toolchain via [rustup.rs](https://rustup.rs/).

### Debug build
Run the following command to trigger a full debug build:
```bash
../building/containerized-build.sh android --dev-build
```

### Release build
1. Configure a signing key by following [these instructions](#configure-signing-key).
2. Run the following command after setting the `ANDROID_CREDENTIALS_DIR` environment variable to the
directory configured in step 1:
```bash
../building/containerized-build.sh android --app-bundle
```

## Build without* the provided container

> __*NOTE:*__ This guide is only supported on Linux and may not work on other platforms, if you are
> using macOS please refer to [macOS build instructions](./docs/BuildInstructions.macos.md)

Building without the provided container requires installing multiple Sdk:s and toolchains, and is
therefore more complex.

> __*\*:*__ A container is still used to build `wireguard-go` for Android since it requires a
> patched version of `go`. See [this patch](https://git.zx2c4.com/wireguard-android/tree/tunnel/tools/libwg-go/goruntime-boottime-over-monotonic.diff)
> for more information.

### Setup build environment
These steps explain how to manually setup the build environment on a Linux system.

#### 1. Install `podman`
Podman is required to build `wireguard-go`. Follow the installation [instructions](https://podman.io/getting-started/installation.html)
for your distribution.

#### 2. Install `protobuf-compiler`
Install a protobuf compiler (version 3 and up), it can be installed on most major Linux distros via
the package name `protobuf-compiler`. An additional package might also be required depending on
Linux distro:
- `protobuf-devel` on Fedora.
- `libprotobuf-dev` on Debian/Ubuntu.

#### 3. Install `gcc`

#### 4. Install Android toolchain

- Install the JDK

  ```bash
  sudo apt install zip openjdk-17-jdk
  ```

- Install the SDK

  The SDK should be placed in a separate directory, like for example `~/android` or `/opt/android`.
  This directory should be exported as the `$ANDROID_HOME` environment variable.

  Note: if `sdkmanager` fails to find the SDK root path, pass the option `--sdk_root=$ANDROID_HOME`
  to the command above.

  ```bash
  cd /opt/android     # Or some other directory to place the Android SDK
  export ANDROID_HOME=$PWD

  wget https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
  unzip commandlinetools-linux-11076708_latest.zip
  ./tools/bin/sdkmanager "platforms;android-33" "build-tools;30.0.3" "platform-tools"
  ```

- Install the NDK

  The NDK should be placed in a separate directory, which can be inside the `$ANDROID_HOME` or in a
  completely separate path. The extracted directory must be exported as the `$ANDROID_NDK_HOME`
  environment variable.

  ```bash
  cd "$ANDROID_HOME"  # Or some other directory to place the Android NDK
  wget https://dl.google.com/android/repository/android-ndk-r27c-linux.zip
  unzip android-ndk-r27c-linux.zip

  cd android-ndk-r27c
  export ANDROID_NDK_HOME="$PWD"
  ```

#### 5. Install and configure Rust toolchain

- Get the latest **stable** Rust toolchain via [rustup.rs](https://rustup.rs/).

- Configure Android cross-compilation targets and set up linker and archiver. This can be done by setting the following
environment variables:

  Add to `~/.bashrc` or equivalent:
  ```
  export NDK_TOOLCHAIN_DIR="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
  export AR_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
  export AR_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/llvm-ar"
  export AR_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
  export AR_i686_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
  export CC_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/aarch64-linux-android26-clang"
  export CC_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi26-clang"
  export CC_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/x86_64-linux-android26-clang"
  export CC_i686_linux_android="$NDK_TOOLCHAIN_DIR/i686-linux-android26-clang"
  export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/aarch64-linux-android26-clang"
  export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi26-clang"
  export CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/i686-linux-android26-clang"
  export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/x86_64-linux-android26-clang"
  ```

- Install Android targets
  ```bash
  ./scripts/setup-rust android
  ```

- (Optional) Run the following to install a git `post-checkout` hook that will automatically
  run the `setup-rust` script when the Rust version specified in the `rust-toolchain.toml` file changes:
  ```bash
  .scripts/setup-rust install-hook
  ```

#### 6. Download wireguard-go-rs submodule
Run the following command to download wireguard-go-rs submodule: `git submodule update --init wireguard-go-rs/libwg/wireguard-go`

### Debug build
Run the following command to build a debug build:
```bash
../android/build.sh --dev-build
```

### Release build
1. Configure a signing key by following [these instructions](#configure-signing-key).
2. Move, copy or symlink the directory from step 1 to [./credentials/](./credentials/) (`<repository>/android/credentials/`).
3. Run the following command to build:
   ```bash
   ../android/build.sh --app-bundle
   ```

## Configure signing key
1. Create a directory to store the signing key, keystore and its configuration:
   ```
   export ANDROID_CREDENTIALS_DIR=/tmp/credentials
   mkdir -p $ANDROID_CREDENTIALS_DIR
   ```

2. Generate a key/keystore named `app-keys.jks` in `ANDROID_CREDENTIALS_DIR` and make sure to write
down the used passwords:
   ```
   keytool -genkey -v -keystore $ANDROID_CREDENTIALS_DIR/app-keys.jks -alias release -keyalg RSA -keysize 4096 -validity 10000
   ```

3. Create a file named `keystore.properties` in `ANDROID_CREDENTIALS_DIR`. Enter the following, but
replace `key-password` and `keystore-password` with the values from step 2:
   ```bash
   keyAlias = release
   keyPassword = key-password
   storePassword = keystore-password
   ```

## Gradle dependency metadata verification lockfile
This lockfile helps ensuring the integrity of the gradle dependencies in the project.

### Update lockfile
When adding or updating dependencies, it's necessary to also update the lockfile. This can be done
in the following way:

1. Run update script:
   ```bash
   ./scripts/update-lockfile.sh
   ```

   If you're on macOS make sure GNU sed is installed. Install with `brew install gnu-sed` and add it to your `PATH` so that it is used instead of the `sed` macOS ships with `PATH="$HOMEBREW_PREFIX/opt/gnu-sed/libexec/gnubin:$PATH"`

2. Check diff before committing.

### Disable during development
This is easiest done by temporarily removing the lockfile:
```bash
rm ./gradle/verification-metadata.xml
```

## Gradle properties
Some gradle properties can be set to simplify development. These are listed below.

### Override version code and version name
To avoid or override the rust based version generation, the `OVERRIDE_VERSION_CODE` and
`OVERRIDE_VERSION_NAME` properties can be set in `local.properties`. For example:
```
OVERRIDE_VERSION_CODE=123
OVERRIDE_VERSION_NAME=1.2.3
```

### Disable version in-app notifications
To disable in-app notifications related to the app version during development or testing,
the `ENABLE_IN_APP_VERSION_NOTIFICATIONS` property can be set in `local.properties`:
```
ENABLE_IN_APP_VERSION_NOTIFICATIONS=false
```

### Run tests highly affected by rate limiting
To avoid being rate limited we avoid running tests sending requests that are highly rate limited too often. If you want to run these tests you can set `enable_highly_rate_limited_tests=true` in `local.properties`. The default value is `false`.

## Reproducible builds

Reproducible builds are a way to verify that the app was built from the exact source code it claims to be built from. When a build is reproducible, compiling the same source code with the same tools will always produce bit-for-bit identical output.

The Mullvad Android app is by default reproducible when built using our build container, as the container ensures a consistent build environment with fixed versions of all tools and dependencies.

When building without the container on Linux systems, reproducibility depends on having the exact same versions of system tools (compilers, build tools, etc) installed. Small differences in tool versions or configurations can lead to different build outputs even when using the same source code.

> **Make sure that the `local.properties` file has not changed keys that affect the reproducibility of the build such as `CARGO_TARGETS` and `ENABLE_IN_APP_VERSION_NOTIFICATIONS`.**

To maximize reproducibility when building without the container:

- Build the app on a **Linux system or virtual machine**.
- Use the exact same versions of all build dependencies as specified in the [root Dockerfile](../building/Dockerfile) and [Android Dockerfile](docker/Dockerfile). This includes for example the Go version and Android SDK and NDK versions.

### How to verify reproducible builds across environments

A simple way to check that a build is reproducible across environments is to build the `fdroid` version of the app with and without the container and comparing the checksums of the produced APKs.

1. Build the app with the container: `../building/containerized-build.sh android --fdroid`
2. Copy the resulting APK to a different folder as it will be overwritten in the following step: `app/build/outputs/apk/ossProd/fdroid/app-oss-prod-fdroid-unsigned.apk fdroid-container.apk`
3. Build the app locally without the container: `./build.sh --fdroid`
4. Compare the checksums of the two APKs: `sha256sum fdroid-container.apk app/build/outputs/apk/ossProd/fdroid/app-oss-prod-fdroid-unsigned.apk`

## Verifying that an official release is reproducible:

1. Obtain the release APK (2025.2-beta1 or newer) from [GitHub releases](https://github.com/mullvad/mullvadvpn-app/releases) or [releases.mullvad.net](https://releases.mullvad.net/android/)
2. Setup your environment so that you can build a [release build ](#release-build) of the app.
3. Make sure you have fetched all tags: `git fetch --tags`
4. Checkout the release tag: `git checkout android/<version>`
5. Build the APK using the containerized script: `../building/containerized-build.sh android`, alternatively build without the container: `./build.sh`. The resuling file is located at `app/build/outputs/apk/ossProd/release/app-oss-prod-release.apk`.
6. Delete the signatures from the two APKs by running `zip -d app-oss-prod-release.apk "META-INF/*"` and `new zip -d MullvadVPN-<version>.apk "META-INF/*"`
7. Compare the checksums of the two APKs: `sha256sum app-oss-prod-release.apk MullvadVPN-<version>.apk`. If the checksums are equal the build is reproducible.

### Troubleshooting reproducibility

If two APKs built from the same commit have different checksums the build is not reproducible. This could be because of either:

1. A build dependency on the local system has the wrong version.
2. There is a bug that breaks the build reproducibility.
3. The APK built is a version prior to 2025.2-beta1, which is the first version that supports reproducible builds.

If you suspect that a bug is causing the build to not be reproducible, please open a Github issue.
