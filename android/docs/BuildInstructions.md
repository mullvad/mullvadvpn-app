# Build instructions

This document explains how to build the Mullvad Android app. For everyday development, follow the
[quick start with Android Studio](#quick-start-with-android-studio).
[Other ways to build](#other-ways-to-build) covers the nix devshell, the container, the command line
(without installing Android Studio) and running cargo directly. [Releases and build configuration](#releases-and-build-configuration)
covers Gradle properties, signing, releases and reproducible builds. To debug the Rust code from
Android Studio, see the [debug instructions](DebugInstructions.md).

## Quick start with Android Studio

1. Clone the repository:
   ```bash
   git clone https://github.com/mullvad/mullvadvpn-app.git
   ```

1. Install [rustup](https://rustup.rs/), a protobuf compiler and a C compiler:
   - macOS (with [Homebrew](https://brew.sh/)): `brew install protobuf`
   - Debian/Ubuntu: `sudo apt install protobuf-compiler libprotobuf-dev gcc`
   - Fedora: `sudo dnf install protobuf-compiler protobuf-devel gcc`

1. Make sure new terminals and Android Studio get the `PATH` change from the rustup installer, for
   example by logging out and back in.

   **On macOS**, start Android Studio from a terminal so the build can use your `PATH` to find
   cargo.

1. From the repository root, install the pinned Rust version and the Android targets:
   ```bash
   ./scripts/setup-rust android
   ```

   **Optionally**, run `./scripts/setup-rust install-hook` and add
   `export MULLVAD_SETUP_PLATFORM=android`, for example to `~/.zshrc` or `~/.bashrc`, to rerun this
   automatically when the Rust version in `rust-toolchain.toml` changes.

1. Install [Android Studio](https://developer.android.com/studio) and complete the setup wizard.

1. Install the NDK. The build doesn't download it, so this is needed even if you already use Android
   Studio: in **Settings > Languages & Frameworks > Android SDK > SDK Tools**, check **Show Package
   Details**, select **NDK (Side by side)** version `27.3.13750724` (the `ndk` version in
   [`gradle/libs.versions.toml`](../gradle/libs.versions.toml)) and click **Apply**.

1. Open the `android` directory in Android Studio and click **Run**.

The first build downloads the other Android SDK packages it needs and compiles the Rust daemon for
all four Android ABIs, so it takes a while (several minutes). To build for fewer ABIs, see
[Build the Rust library for fewer ABIs](#build-the-rust-library-for-fewer-abis).

### Enable command line builds

Optionally, you can also build from the command line after the quick start. Android Studio stores
the SDK location in `android/local.properties`, so Gradle only needs a JDK. You can use the one
bundled with Android Studio by setting `JAVA_HOME` to
`"/Applications/Android Studio.app/Contents/jbr/Contents/Home"` on macOS, or on Linux
`/usr/local/android-studio/jbr` or `/opt/android-studio/jbr` depending on where you unpacked it.
You can then build a debug APK by running `./gradlew debug` from the `android` directory, which
will end up as: `app/build/outputs/apk/ossProd/debug/app-oss-prod-debug.apk`

## Other ways to build

The app is built with Gradle, which also builds the Rust daemon (`mullvad-daemon`) as a native
library through the [rust-android-gradle](https://github.com/mullvad/rust-android-gradle) plugin.
The plugin sets up the NDK toolchain for cargo, so building through Gradle doesn't need any NDK
environment variables. For release builds it's strongly recommended to use the
[container](#build-using-the-container), as it ensures the correct build environment.

### Build using nix devshell
This is supported on Linux (x86_64) as well as macOS (x86_64 and aarch64).

1. Install the nix package manager by following the [official instructions](https://nixos.org/download/).
1. Enable the experimental `nix-command` and `flake` features by following [these instructions](https://nixos.wiki/wiki/flakes).
1. Build a debug APK:
   ```bash
   nix develop .#android -c build
   ```
   Artifact path: `app/build/outputs/apk/ossProd/debug/app-oss-prod-debug.apk`

To run another command in the devshell, pass it after `-c`, for example
`nix develop .#android -c ./gradlew tasks`, or run `nix develop .#android` to enter an interactive
shell.

### Build using the container
This is primarily supported on Linux, and might work on macOS and Windows to varying degrees.

1. Install [podman](https://podman.io/getting-started/installation.html) and make sure it's
   configured to run in rootless mode. To use another container runner such as
   [docker](https://docs.docker.com/get-started/) (which might work better on e.g. macOS),
   set the `CONTAINER_RUNNER` environment variable.
1. Build a debug APK from the repository root:
   ```bash
   ./building/containerized-build.sh android
   ```
   Artifact path: `android/app/build/outputs/apk/ossProd/debug/app-oss-prod-debug.apk`

To run another Gradle task in the container, pass it after `android`, for example
`./building/containerized-build.sh android tasks`.

### Build using the command line (without installing Android Studio)
This works on Linux and macOS.

1. Clone the repository and install rustup, a protobuf compiler, a C compiler, the pinned Rust
   version and the Android targets as in steps 1 to 4 of the
   [quick start](#quick-start-with-android-studio).
1. Install a JDK:
   - macOS: `brew install openjdk@21`, then set `JAVA_HOME` to
     `$(brew --prefix openjdk@21)/libexec/openjdk.jdk/Contents/Home`.
   - Debian/Ubuntu: `sudo apt install zip openjdk-21-jdk`
   - Fedora: `sudo dnf install java-21-openjdk-devel`
1. Install the Android development tooling:
   - Install the Android command line tools as described in the [sdkmanager docs](https://developer.android.com/tools/sdkmanager).
   - Set the `ANDROID_HOME` environment variable, for example in `~/.zshrc` or `~/.bashrc`.
   - Accept the SDK licenses and install the NDK:
      ```bash
      yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses
      "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" "ndk;27.3.13750724"
      ```
   The build downloads the SDK platform and build tools it needs.
1. Build a debug APK from the `android` directory:
   ```bash
   ./gradlew debug
   ```
   Artifact path: `app/build/outputs/apk/ossProd/debug/app-oss-prod-debug.apk`

### Build the Rust library directly with cargo

Gradle builds the Rust library through the plugin, so this is only needed when running cargo
yourself against an Android target, for example to build or lint the library without Gradle. Cargo
then needs the NDK toolchain through the environment variables below. The
[nix devshell](#build-using-nix-devshell) and the [container](#build-using-the-container) already
set them.

To run cargo in your own shell, install the Android Rust targets with `./scripts/setup-rust android`
and set the variables, for example in `~/.zshrc` or `~/.bashrc`. Also set `ANDROID_HOME` to an SDK
with the NDK installed. If you set it up as in the [quick start](#quick-start-with-android-studio),
that's `~/Library/Android/sdk` on macOS and `~/Android/Sdk` on Linux. The NDK toolchain directory
is `darwin-x86_64` on macOS (also on Apple silicon) and `linux-x86_64` on Linux, which the `uname`
call picks. The versions must match [`gradle/libs.versions.toml`](../gradle/libs.versions.toml):
`27.3.13750724` is the `ndk` version and the `28` in the compiler names is the `min-sdk` version.

```bash
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.3.13750724"
export NDK_TOOLCHAIN_DIR="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin"
export AR_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_i686_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export CC_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/aarch64-linux-android28-clang"
export CC_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi28-clang"
export CC_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/x86_64-linux-android28-clang"
export CC_i686_linux_android="$NDK_TOOLCHAIN_DIR/i686-linux-android28-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/aarch64-linux-android28-clang"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi28-clang"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/x86_64-linux-android28-clang"
export CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/i686-linux-android28-clang"
```

Then run cargo from the repository root as usual, for example:
```bash
cargo build --target aarch64-linux-android --package mullvad-jni
```

## Releases and build configuration

### Gradle properties
Some Gradle properties can be set to simplify development, for the full list see [`gradle.properties`](../gradle.properties).
In order to override them, add the properties in `<USER_GRADLE_HOME>/gradle.properties`. See the
[gradle documentation](https://docs.gradle.org/current/userguide/build_environment.html#sec:project_properties)
for more info of the prioritization of properties.

#### Build the Rust library for fewer ABIs
By default the Rust library is built for all four Android ABIs. To speed up development builds,
build it only for the ABI of your device or emulator, for example:
```
mullvad.app.build.cargo.targets=arm64
```
Use `arm64` for most phones and for emulators on Apple silicon, and `x86_64` for emulators on Intel
and AMD machines. The available values are `arm`, `arm64`, `x86` and `x86_64`. Remove the property
before making release builds, since the app then only works on the listed ABIs.

#### Override version code and version name
To avoid or override the rust based version generation, the `mullvad.app.config.override.versionCode` and
`mullvad.app.config.override.versionName` properties can be set:
```
mullvad.app.config.override.versionCode=123
mullvad.app.config.override.versionName=1.2.3
```

#### Disable version in-app notifications
To disable in-app notifications related to the app version during development or testing,
the `mullvad.app.config.inAppVersionNotifications.enable` property can be set:
```
mullvad.app.config.inAppVersionNotifications.enable=false
```

#### Run tests highly affected by rate limiting
To avoid being rate limited we avoid running tests sending requests that are highly rate limited
too often. If you want to run these tests you can override the
`mullvad.test.e2e.config.runHighlyRateLimitedTests` Gradle properties. The default value is `false`.

### Gradle dependency metadata verification lockfile
This lockfile helps ensure the integrity of the Gradle dependencies in the project.

#### Update lockfile
When adding or updating dependencies, it's necessary to also update the lockfile. This can be done
in the following way:

1. Run update script:
   ```bash
   ./scripts/lockfile -u
   ```

   If you're on macOS make sure GNU sed is installed. Install with `brew install gnu-sed` and add it to your `PATH` so that it is used instead of the `sed` macOS ships with `PATH="$HOMEBREW_PREFIX/opt/gnu-sed/libexec/gnubin:$PATH"`

1. Check diff before committing.

#### Disable during development
This is easiest done by temporarily removing the lockfile:
```bash
rm ./gradle/verification-metadata.xml
```

### Release build
1. Configure a signing key by following [these instructions](#configure-signing-key).
1. Build the release artifacts in the [container](#build-using-the-container) from the repository
   root:
   ```bash
   ./building/containerized-build.sh android fullRelease
   ```
   Without the container, run `./gradlew fullRelease` from the `android` directory instead.
1. Sign the release artifacts using apksigner with the following command:
   ```bash
   apksigner sign --ks app-keys.jks (MullvadVPN)(version)(.apk|.aab)
   ```

### Configure signing key
Generate a key/keystore named `app-keys.jks` and make sure to write
down the used passwords:
   ```bash
   keytool -genkey -v -keystore app-keys.jks -alias release -keyalg RSA -keysize 4096 -validity 10000
   ```

### Creating an alpha release

Run the [prepare-release.sh](../scripts/prepare-release.sh) script from the `android` directory
with the alpha version you intend to create as an argument, for example `2026.13-alpha1`. The
prepare script will download the latest relay list and update the version files, and add as commits.

```bash
./scripts/prepare-release.sh 2026.13-alpha1
```

Continue by following the instructions provided by the script.

### Reproducible builds

Reproducible builds are a way to verify that the app was built from the exact source code it claims to be built from. When a build is reproducible, compiling the same source code with the same tools will always produce bit-for-bit identical output.

The Mullvad Android app is by default reproducible when built using our build container, as the container ensures a consistent build environment with fixed versions of all tools and dependencies.

When building without the container on Linux systems, reproducibility depends on having the exact same versions of system tools (compilers, build tools, etc.) installed. Small differences in tool versions or configurations can lead to different build outputs even when using the same source code.

> **Make sure that any `gradle.properties` has not changed or been overridden it will affect the reproducibility of the build such as changing `mullvad.app.build.cargo.targets` and `mullvad.app.config.inAppVersionNotifications.enable`.**

To maximize reproducibility when building without the container:

- Build the app on a **Linux system or virtual machine**.
- Use the exact same versions of all build dependencies as specified in the [Android Dockerfile](../docker/Dockerfile)
  and the setup scripts in [building](../../building) that it runs. This includes for example the Rust toolchain,
  protobuf compiler, and Android SDK and NDK versions.

#### How to verify reproducible builds across environments

A simple way to check that a build is reproducible across environments is to build the release version of the app with and without the container and comparing the checksums of the produced APKs. Run these steps from the `android` directory.

1. Build the app with the container: `../building/containerized-build.sh android assembleOssProdRelease`
1. Copy the resulting APK to a different folder as it will be overwritten in the following step: `cp app/build/outputs/apk/ossProd/release/app-oss-prod-release-unsigned.apk release-container.apk`
1. Build the app locally without the container: `./gradlew assembleOssProdRelease`
1. Compare the checksums of the two APKs: `sha256sum release-container.apk app/build/outputs/apk/ossProd/release/app-oss-prod-release-unsigned.apk`

### Verifying that an official release is reproducible

1. Obtain the release APK (`2025.2-beta1` or newer) from [GitHub releases](https://github.com/mullvad/mullvadvpn-app/releases)
1. Checkout the release tag: `git checkout android/<version>`
1. Build a release build using our [build instructions](#release-build)
1. Delete the signatures from the two APKs by running `zip -d app-oss-prod-release.apk "META-INF/*"` and `zip -d MullvadVPN-<version>.apk "META-INF/*"`
1. Compare the checksums of the two APKs: `sha256sum app-oss-prod-release.apk MullvadVPN-<version>.apk`. If the checksums are equal the build is reproducible.

#### Troubleshooting reproducibility

If two APKs built from the same commit have different checksums the build is not reproducible. This could be because of either:

1. A build dependency on the local system has the wrong version.
1. There is a bug that breaks the build reproducibility.
1. The APK built is a version prior to `2025.2-beta1`, which is the first version that supports reproducible builds.

If you suspect that a bug is causing the build to not be reproducible, please open a GitHub issue.
