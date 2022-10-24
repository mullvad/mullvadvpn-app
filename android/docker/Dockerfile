# To build the image:
# podman build . -t mullvadvpn-app-build-android
#
# To build using the image:
# podman run --rm \
#     -v /path/to/container_cache/target:/root/.cargo/target:Z \
#     -v /path/to/container_cache/registry:/root/.cargo/registry:Z \
#     -v /path/to/container_cache/gradle:/root/.gradle:Z \
#     -v /path/to/repository_root:/build:Z \
#     mullvadvpn-app-build-android ./build-apk.sh --dev-build --no-docker
#
# See the base image Dockerfile in the repository root (../../Dockerfile)
# for more information.

# === Base image (built from: ../../Dockerfile) ===
# NOTE: When bumping the base image, it's important to keep the go patch applied
# in this image in sync with the go version used in the base image. An appropriate
# patch for a given go version can be identified by checking the wireguard-android
# repo: https://git.zx2c4.com/wireguard-android/tree/tunnel/tools/libwg-go.
# It's also important to keep the go path in sync.
FROM ghcr.io/mullvad/mullvadvpn-app-build@sha256:b539c83c4336246fef1c324242b49b0fd35fb0839dffd2b9c9fc8361e629a40c

# === Metadata ===
LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app Android build container"
LABEL org.opencontainers.image.licenses=GPL-3.0

# === Define toolchain versions and paths ===

ENV SDK_VERSION=platforms;android-33 \
    BUILD_TOOLS_VERSION=build-tools;30.0.2

# SDK tools
ENV SDK_TOOLS_VERSION=4333796 \
    SDK_TOOLS_SHA256_CHECKSUM=92ffee5a1d98d856634e8b71132e8a95d96c83a63fde1099be3d86df3106def9

# NDK and checksum from: https://github.com/android/ndk/wiki/Unsupported-Downloads
ENV NDK_VERSION=r20b \
    NDK_SHA1_CHECKSUM=d903fdf077039ad9331fb6c3bee78aa46d45527b \
    MIN_SDK_VERSION=21

ENV ANDROID_HOME=/opt/android
ENV ANDROID_NDK_HOME=${ANDROID_HOME}/android-ndk-${NDK_VERSION}
ENV NDK_TOOLCHAIN_DIR=${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin
ENV GRADLE_USER_HOME=/root/.gradle

# Rust cross-compilation for: aarch64
ENV AR_aarch64_linux_android=${NDK_TOOLCHAIN_DIR}/aarch64-linux-android-ar \
    CC_aarch64_linux_android=${NDK_TOOLCHAIN_DIR}/aarch64-linux-android${MIN_SDK_VERSION}-clang \
    CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=${NDK_TOOLCHAIN_DIR}/aarch64-linux-android${MIN_SDK_VERSION}-clang

# Rust cross-compilation for: armv7
ENV AR_armv7_linux_androideabi=${NDK_TOOLCHAIN_DIR}/arm-linux-androideabi-ar \
    CC_armv7_linux_androideabi=${NDK_TOOLCHAIN_DIR}/armv7a-linux-androideabi${MIN_SDK_VERSION}-clang \
    CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=${NDK_TOOLCHAIN_DIR}/armv7a-linux-androideabi${MIN_SDK_VERSION}-clang

# Rust cross-compilation for: i686
ENV AR_i686_linux_android=${NDK_TOOLCHAIN_DIR}/i686-linux-android-ar \
    CC_i686_linux_android=${NDK_TOOLCHAIN_DIR}/i686-linux-android${MIN_SDK_VERSION}-clang \
    CARGO_TARGET_I686_LINUX_ANDROID_LINKER=${NDK_TOOLCHAIN_DIR}/i686-linux-android${MIN_SDK_VERSION}-clang

# Rust cross-compilation for: x86_64
ENV AR_x86_64_linux_android=${NDK_TOOLCHAIN_DIR}/x86_64-linux-android-ar \
    CC_x86_64_linux_android=${NDK_TOOLCHAIN_DIR}/x86_64-linux-android${MIN_SDK_VERSION}-clang \
    CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER=${NDK_TOOLCHAIN_DIR}/x86_64-linux-android${MIN_SDK_VERSION}-clang

# === Install/set up the image ===

RUN apt-get update -y && apt-get install -y \
    file \
    gpg \
    make \
    python \
    software-properties-common \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install JDK
COPY adopt-openjdk-apt-key.pgp /tmp/adopt-openjdk-apt-key.pgp
RUN apt-key add < /tmp/adopt-openjdk-apt-key.pgp && \
    add-apt-repository -y https://adoptopenjdk.jfrog.io/adoptopenjdk/deb/ && \
    apt-get update && \
    apt-get install -y adoptopenjdk-8-hotspot && \
    rm /tmp/adopt-openjdk-apt-key.pgp && rm -rf /var/lib/apt/lists/*

# Install base Android SDK tools
RUN curl -sfLo /tmp/sdk-tools.zip https://dl.google.com/android/repository/sdk-tools-linux-${SDK_TOOLS_VERSION}.zip && \
    echo "$SDK_TOOLS_SHA256_CHECKSUM /tmp/sdk-tools.zip" | sha256sum -c && \
    mkdir -p $ANDROID_HOME && \
    unzip -q /tmp/sdk-tools.zip -d $ANDROID_HOME && \
    rm /tmp/sdk-tools.zip

# Install Android SDK
RUN yes | $ANDROID_HOME/tools/bin/sdkmanager $SDK_VERSION $BUILD_TOOLS_VERSION "platform-tools"

# Install Android NDK
RUN curl -sfLo /tmp/ndk.zip https://dl.google.com/android/repository/android-ndk-${NDK_VERSION}-linux-x86_64.zip && \
    echo "$NDK_SHA1_CHECKSUM /tmp/ndk.zip" | sha1sum -c && \
    unzip -q /tmp/ndk.zip -d $ANDROID_HOME && \
    rm /tmp/ndk.zip

# Apply go patch required for Android
# See the comment above the FROM instruction for more information.
COPY goruntime-boottime-over-monotonic.diff /tmp/goruntime-boottime-over-monotonic.diff
RUN patch -p1 -f -N -r- -d /usr/local/go < /tmp/goruntime-boottime-over-monotonic.diff && \
    rm /tmp/goruntime-boottime-over-monotonic.diff

# Add rust targets
RUN rustup target add x86_64-linux-android i686-linux-android aarch64-linux-android armv7-linux-androideabi

WORKDIR /build
