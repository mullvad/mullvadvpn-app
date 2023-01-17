# Build instructions

This document aims to explain how to build the Mullvad Android app. It's strongly recommended and
primarily supported to build the app using the provided container, as it ensures the correct build
environment.

## Build with provided container (recommended)

This can easily be achieved by running the [containerized-build.sh](../building/containerized-build.sh)
script, which helps using the correct tag and mounting volumes. The script relies on [podman](https://podman.io/getting-started/installation.html)
by default, however another container runner such as [docker](https://docs.docker.com/get-started/)
can be used by setting the `CONTAINER_RUNNER` environment variable.

### Debug build
Run the following command to trigger a full debug build:
```bash
../building/containerized-build.sh android --dev-build
```

### Release build
1. Configure a signing key by following [these instructions](#configure-signing-key).
2. Run the following command after setting the `ANDROID_CREDENTIALS_DIR` enviroment variable to the
directory configured in step 1:
```bash
../building/containerized-build.sh android --app-bundle
```

## Build without* the provided container (not recommended)

Building without the provided container requires installing multiple Sdk:s and toolchains, and is
therefore not recommended.

*: A container is still used to build `wireguard-go`.

### Setup build enviroment
These steps explain how to manually setup the build environment on a Linux system.

#### 1. Install `docker`
Docker is required to build `wireguard-go`. Follow the installation [instructions](https://docs.docker.com/engine/install/debian/)
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
  sudo apt install zip openjdk-11-jdk
  ```

- Install the SDK

  The SDK should be placed in a separate directory, like for example `~/android` or `/opt/android`.
  This directory should be exported as the `$ANDROID_HOME` environment variable.

  Note: if `sdkmanager` fails to find the SDK root path, pass the option `--sdk_root=$ANDROID_HOME`
  to the command above.

  ```bash
  cd /opt/android     # Or some other directory to place the Android SDK
  export ANDROID_HOME=$PWD

  wget https://dl.google.com/android/repository/commandlinetools-linux-8512546_latest.zip
  unzip commandlinetools-linux-6609375_latest.zip
  ./tools/bin/sdkmanager "platforms;android-33" "build-tools;30.0.2" "platform-tools"
  ```

- Install the NDK

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

#### 5. Install and configure Rust toolchain

- Get the latest **stable** Rust toolchain via [rustup.rs](https://rustup.rs/).

- Configure Android cross-compilation targets. This can be done by setting the following
environment variables:
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

- Install Android targets
  ```bash
  rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
  ```

- Set up cargo to use the correct linker and archiver

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

### Debug build
Run the following command to build a debug build:
```bash
../build-apk.sh --dev-build
```

### Release build
1. Configure a signing key by following [these instructions](#configure-signing-key).
2. Move, copy or symlink the directory from step 1 to [./credentials/](./credentials/) (`<repository>/android/credentials/`).
3. Run the following command to build:
   ```bash
   ../build-apk.sh --app-bundle
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

1. Run update script (requires `podman`):
   ```bash
   ./scripts/update-lockfile.sh
   ```
2. Check diff before committing.

### Disable during development
This is easiest done by temporarily removing the lockfile:
```bash
rm ./gradle/verification-metadata.xml
```
