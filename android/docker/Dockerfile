# To build the image (executed from the repository root):
# podman build -f android/docker/Dockerfile . -t mullvadvpn-app-build-android
#
# To build using the image:
# podman run --rm \
#     -v $CARGO_TARGET_VOLUME_NAME:/cargo-target:Z \
#     -e CARGO_TARGET_DIR=/cargo-target/target \
#     -v $CARGO_REGISTRY_VOLUME_NAME:/root/.cargo/registry:Z \
#     -v $GRADLE_CACHE_VOLUME_NAME:/root/.gradle:Z \
#     -v /path/to/repository_root:/build:Z \
#     mullvadvpn-app-build-android ./android/gradlew -p android debug

# Base image: Debian trixie (13.7-slim)
FROM --platform=linux/amd64 debian@sha256:a99cfc517144bc59b1978475ec53b46ecabec7e43635402ee5b77cc54cd1b20a

# === Metadata ===
LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app Android build container"
LABEL org.opencontainers.image.licenses=GPL-3.0-or-later

# === Configure snapshot ===

ARG DEBIAN_SNAPSHOT=20260927T000000Z
COPY building/setup-debian-snapshot.sh /tmp/
RUN /tmp/setup-debian-snapshot.sh && rm /tmp/setup-debian-snapshot.sh

# === Define toolchain versions ===

ARG SDK_VERSION=android-37.0 \
    BUILD_TOOLS_VERSION=37.0.0

# Command line tools and checksum from: https://developer.android.com/studio#command-line-tools-only
ARG COMMAND_LINE_TOOLS_VERSION=15859902 \
    COMMAND_LINE_TOOLS_SHA256_CHECKSUM=4e4c464f145a7512b57d088ac6c278c03c9eea610886b35a5e0804e74eedf583

# NDK and checksum from: https://github.com/android/ndk/wiki#supported-downloads
ARG NDK_VERSION_NAME=r27d \
    NDK_SHA1_CHECKSUM=22105e410cf29afcf163760cc95522b9fb981121 \
    MIN_SDK_VERSION=28 \
    NDK_VERSION=27.3.13750724

# === Define paths and environment ===

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

ENV JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64

# Make adb, apk signer, apk analyzer and other tools accessible
ENV PATH="\
${JAVA_HOME}/bin\
:${ANDROID_SDK_ROOT}/platform-tools\
:${ANDROID_SDK_ROOT}/build-tools/${BUILD_TOOLS_VERSION}\
:${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin\
:${PATH}"

# === Install base packages ===

RUN apt-get update -y && apt-get install -y \
    file \
    git \
    curl \
    jq \
    make \
    gcc \
    gpg \
    unzip \
    ca-certificates-java \
    openjdk-21-jdk-headless \
    tidy \
    pcscd \
    rclone \
    yubico-piv-tool \
    ykcs11 \
    protobuf-compiler \
    libprotobuf-dev \
    && apt-get install -y --no-install-recommends fdroidserver \
    && rm -rf /var/lib/apt/lists/*

# === Install rust ===

COPY building/install-rust.sh rust-toolchain.toml /tmp/rust/
RUN /tmp/rust/install-rust.sh android && rm -r /tmp/rust
ENV PATH=/root/.cargo/bin:$PATH

# === Install SBOM tools ===

RUN --mount=type=bind,source=building/install-cyclonedx.sh,target=/tmp/install-cyclonedx.sh,z \
    /tmp/install-cyclonedx.sh

# === Install Android packages ===

# Install Android command line tools
RUN curl -sfLo /tmp/cmdline-tools.zip https://dl.google.com/android/repository/commandlinetools-linux-${COMMAND_LINE_TOOLS_VERSION}_latest.zip && \
    echo "$COMMAND_LINE_TOOLS_SHA256_CHECKSUM /tmp/cmdline-tools.zip" | sha256sum -c && \
    unzip -q /tmp/cmdline-tools.zip -d /tmp/ && \
    mkdir -p $ANDROID_SDK_ROOT/cmdline-tools && \
    mv /tmp/cmdline-tools $ANDROID_SDK_ROOT/cmdline-tools/latest && \
    rm /tmp/cmdline-tools.zip

# Install Android SDK
RUN yes | $ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager \
    "platforms;$SDK_VERSION" \
    "build-tools;$BUILD_TOOLS_VERSION" \
    "platform-tools"

# Install Android NDK
RUN mkdir -p ${ANDROID_SDK_ROOT}/ndk && \
    curl -sfLo /tmp/ndk.zip https://dl.google.com/android/repository/android-ndk-${NDK_VERSION_NAME}-linux.zip && \
    echo "$NDK_SHA1_CHECKSUM /tmp/ndk.zip" | sha1sum -c && \
    unzip -q /tmp/ndk.zip -d /tmp && \
    mv /tmp/android-ndk-${NDK_VERSION_NAME} ${ANDROID_NDK_HOME} && \
    rm /tmp/ndk.zip

# === Configure for signing ===

COPY android/docker/pkcs11_java.cfg /usr/local/etc/pkcs11_java.cfg
# Keep ykcs11 at the path used by pkcs11_java.cfg and existing override configs
RUN ln -s /usr/lib/x86_64-linux-gnu/libykcs11.so.2 /usr/local/lib/libykcs11.so.2

# There is no polkit daemon in the container to authorize pcscd clients
RUN echo 'PCSCD_ARGS="--disable-polkit"' >> /etc/default/pcscd

WORKDIR /build
