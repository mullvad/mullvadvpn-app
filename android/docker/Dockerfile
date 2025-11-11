# To build the image (executed from this Dockerfile directory):
# podman build . -t mullvadvpn-app-build-android
#
# To build using the image:
# podman run --rm \
#     -v $CARGO_TARGET_VOLUME_NAME:/cargo-target:Z \
#     -v $CARGO_REGISTRY_VOLUME_NAME:/root/.cargo/registry:Z \
#     -v $GRADLE_CACHE_VOLUME_NAME:/root/.gradle:Z \
#     -v $ANDROID_CREDENTIALS_DIR:/build/android/credentials:Z \
#     -v /path/to/repository_root:/build:Z \
#     mullvadvpn-app-build-android ./android/build.sh --dev-build
#
# See the base image Dockerfile in the repository root (../../Dockerfile)
# for more information.

# === Base image (built from: ../../Dockerfile) ===
# NOTE: When bumping the base image, it's important to keep the go patch applied
# in this image in sync with the go version used in the base image. An appropriate
# patch for a given go version can be identified by checking the wireguard-android
# repo: https://git.zx2c4.com/wireguard-android/tree/tunnel/tools/libwg-go.
# It's also important to keep the go path in sync.
FROM ghcr.io/mullvad/mullvadvpn-app-build:dbc170b2c5

# === Metadata ===
LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app Android build container"
LABEL org.opencontainers.image.licenses=GPL-3.0

# === Define toolchain versions and paths ===

ARG SDK_VERSION=platforms;android-36 \
    BUILD_TOOLS_VERSION=build-tools;36.0.0

# Command line tools and checksum from: https://developer.android.com/studio#command-line-tools-only
ARG COMMAND_LINE_TOOLS_VERSION=13114758 \
    COMMAND_LINE_TOOLS_SHA256_CHECKSUM=7ec965280a073311c339e571cd5de778b9975026cfcbe79f2b1cdcb1e15317ee

# NDK and checksum from: https://github.com/android/ndk/wiki#supported-downloads
ARG NDK_VERSION_NAME=r27d \
    NDK_SHA1_CHECKSUM=22105e410cf29afcf163760cc95522b9fb981121 \
    MIN_SDK_VERSION=26 \
    NDK_VERSION=27.3.13750724

ENV ANDROID_SDK_ROOT=/opt/android
# ANDROID_HOME is kept for backwards compatibility
ENV ANDROID_HOME=$ANDROID_SDK_ROOT
ENV ANDROID_NDK_HOME=${ANDROID_SDK_ROOT}/ndk/${NDK_VERSION}
ENV NDK_TOOLCHAIN_DIR=${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin
ENV GRADLE_USER_HOME=/root/.gradle

# Rust cross-compilation for: aarch64
ENV AR_aarch64_linux_android=${NDK_TOOLCHAIN_DIR}/llvm-ar \
    CC_aarch64_linux_android=${NDK_TOOLCHAIN_DIR}/aarch64-linux-android${MIN_SDK_VERSION}-clang \
    CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=${NDK_TOOLCHAIN_DIR}/aarch64-linux-android${MIN_SDK_VERSION}-clang

# Rust cross-compilation for: armv7
ENV AR_armv7_linux_androideabi=${NDK_TOOLCHAIN_DIR}/llvm-ar \
    CC_armv7_linux_androideabi=${NDK_TOOLCHAIN_DIR}/armv7a-linux-androideabi${MIN_SDK_VERSION}-clang \
    CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=${NDK_TOOLCHAIN_DIR}/armv7a-linux-androideabi${MIN_SDK_VERSION}-clang

# Rust cross-compilation for: i686
ENV AR_i686_linux_android=${NDK_TOOLCHAIN_DIR}/llvm-ar \
    CC_i686_linux_android=${NDK_TOOLCHAIN_DIR}/i686-linux-android${MIN_SDK_VERSION}-clang \
    CARGO_TARGET_I686_LINUX_ANDROID_LINKER=${NDK_TOOLCHAIN_DIR}/i686-linux-android${MIN_SDK_VERSION}-clang

# Rust cross-compilation for: x86_64
ENV AR_x86_64_linux_android=${NDK_TOOLCHAIN_DIR}/llvm-ar \
    CC_x86_64_linux_android=${NDK_TOOLCHAIN_DIR}/x86_64-linux-android${MIN_SDK_VERSION}-clang \
    CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER=${NDK_TOOLCHAIN_DIR}/x86_64-linux-android${MIN_SDK_VERSION}-clang

# Set up python3 path for the rust gradle plugin
ENV RUST_ANDROID_GRADLE_PYTHON_COMMAND=/usr/bin/python3

# Make adb and other platform tools accessible
ENV PATH="${ANDROID_SDK_ROOT}/platform-tools:${PATH}"

# === Install/set up the image ===

RUN apt-get update -y && apt-get install -y \
    file \
    gpg \
    make \
    python \
    software-properties-common \
    unzip \
    ca-certificates-java \
    openjdk-17-jdk \
    tidy \
    && rm -rf /var/lib/apt/lists/*

# Set default java version
RUN update-alternatives --set java  /usr/lib/jvm/java-17-openjdk-amd64/bin/java
RUN update-alternatives --set javac /usr/lib/jvm/java-17-openjdk-amd64/bin/javac

# Install Android command line tools
RUN curl -sfLo /tmp/cmdline-tools.zip https://dl.google.com/android/repository/commandlinetools-linux-${COMMAND_LINE_TOOLS_VERSION}_latest.zip && \
    echo "$COMMAND_LINE_TOOLS_SHA256_CHECKSUM /tmp/cmdline-tools.zip" | sha256sum -c && \
    unzip -q /tmp/cmdline-tools.zip -d /tmp/ && \
    mkdir -p $ANDROID_SDK_ROOT/cmdline-tools && \
    mv /tmp/cmdline-tools $ANDROID_SDK_ROOT/cmdline-tools/latest && \
    rm /tmp/cmdline-tools.zip

# Install Android SDK
RUN yes | $ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager $SDK_VERSION $BUILD_TOOLS_VERSION "platform-tools"

# Install Android NDK
RUN mkdir -p ${ANDROID_SDK_ROOT}/ndk && \
    curl -sfLo /tmp/ndk.zip https://dl.google.com/android/repository/android-ndk-${NDK_VERSION_NAME}-linux.zip && \
    echo "$NDK_SHA1_CHECKSUM /tmp/ndk.zip" | sha1sum -c && \
    unzip -q /tmp/ndk.zip -d /tmp && \
    mv /tmp/android-ndk-${NDK_VERSION_NAME} ${ANDROID_NDK_HOME} && \
    rm /tmp/ndk.zip

# Apply go patch required for Android
# See the comment above the FROM instruction for more information.
COPY goruntime-boottime-over-monotonic.diff /tmp/goruntime-boottime-over-monotonic.diff
RUN patch -p1 -f -N -r- -d /usr/local/go < /tmp/goruntime-boottime-over-monotonic.diff && \
    rm /tmp/goruntime-boottime-over-monotonic.diff

# Add rust targets
RUN rustup target add x86_64-linux-android i686-linux-android aarch64-linux-android armv7-linux-androideabi

WORKDIR /build
