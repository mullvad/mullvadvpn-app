# Docker image to build the Android

This folder contains the necessary files to create a Docker image that can be used to build the
Android app without having to configure the local environment first. The image contains the
Android SDK, the Android NDK, the patched Go compiler, the Rust compiler and the necessary
configuration to allow the app to build.

To build the image, the following command can be used while inside this directory:

```
docker build -t mullvad/mullvadvpn-app-android .
```

After the image has been built, it can be used to build the Android app. Given that the source code
[repository](https://github.com/mullvad/mullvadvpn-app) was checkout out on
`/home/user/mullvadvpn-app`, the following command will build the APK there:

```
docker run \
    --rm \
    -it \
    --name mullvad-android-build \
    -v /home/user/mullvadvpn-app:/project \
    -w /project \
    mullvad/mullvadvpn-app-android
```

The container can be configured to build the native libraries for a subset of the supported
architectures by setting the `ARCHITECTURES` environment variable. The supported architecuters are:

- 64-bit ARMv8: `aarch64`
- 32-bit ARMv7: `armv7`
- 64-bit x86-64: `x86_64`
- 32-bit x86: `i686`

The example below builds only for 64-bit ARM and x86-64:

```
docker run \
    --rm \
    -it \
    -e ARCHITECTURES="aarch64 x86_64"
    --name mullvad-android-build \
    -v /home/user/mullvadvpn-app:/project \
    -w /project \
    mullvad/mullvadvpn-app-android
```

## Speeding up the build with caches

To speed up the build, some cache files for Cargo and Gradle can be reused between builds. There are
two options to configure reusing those files: creating directories for them in the repository or
creating separate volumes.

### Using extra directories inside the repository

Two directories can be created inside the repository, `.gradle-home` and `.cargo-home`. When
building, they can be configured to be used by the build using environment variables
(`GRADLE_USER_HOME` and `CARGO_HOME`). The following command shows how to run the build container
using the extra directories:

```
docker run \
    --rm \
    -it \
    --name mullvad-android-build \
    -v /home/user/mullvadvpn-app:/project \
    -e GRADLE_USER_HOME=/project/.gradle-home \
    -e CARGO_HOME=/project/.cargo-home \
    -w /project \
    mullvad/mullvadvpn-app-android
```

### Using Docker volumes

Some extra volumes can be used to cache Cargo and Gradle data. The following commands set up those
volumes and runs the build container using them:

```
# Run these commands once
docker volume create cargo-git
docker volume create cargo-registry
docker volume create gradle-cache

# Run this command every time to build
docker run \
    --rm \
    -it \
    --name mullvad-android-build \
    -v /home/user/mullvadvpn-app:/project \
    -v cargo-git:/root/.cargo/git \
    -v cargo-registry:/root/.cargo/registry \
    -v gradle-cache:/root/.gradle \
    -w /project \
    mullvad/mullvadvpn-app-android
```
