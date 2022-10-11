These instructions are for building the app for Android **under Linux**.

# Set up build environment

These instructions are probably not complete. If you find something more that needs installing
on your platform please submit an issue or a pull request.

- Docker is required to build `wireguard-go`. Follow the
  [instructions](https://docs.docker.com/engine/install/debian/) for your distribution.

- Install a protobuf compiler (version 3 and up), it can be installed on most major Linux distros
  via the package name `protobuf-compiler`. An additional package might also be required depending
  on Linux distro:
  - `protobuf-devel` on Fedora.
  - `libprotobuf-dev` on Debian/Ubuntu.

- Install `gcc`

## Android toolchain

- Install the JDK

    ```bash
    sudo apt install zip default-jdk
    ```

- Install the SDK

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

## Configuring Rust

- Get the latest **stable** Rust toolchain via [rustup.rs](https://rustup.rs/).

### Install Rust Android targets

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

### Set up cargo to use the correct linker and archiver

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

# Building a debug build

Run `build-apk.sh` with `--dev-build` to build the Rust daemon and the UI in debug mode and sign the
APK with automatically generated debug keys:
```bash
../build-apk.sh --dev-build
```

If the above fails with an error related to compression, try allowing more memory to the JVM:
```bash
echo "org.gradle.jvmargs=-Xmx4608M" >> ~/.gradle/gradle.properties
./gradlew --stop
```

# Building a release build

## Configure signing key

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

## Building and packaging the app

Running the `build-apk.sh` script will build the necessary Rust daemon for all supported ABIs and
build the final APK:
```bash
../build-apk.sh
```
