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

To speed up the build, some extra volumes can be used to cache Cargo and Gradle data. The following
commands set up those volumes and runs the build container using them:

```
docker volume create cargo-git
docker volume create cargo-registry
docker volume create gradle-cache

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
