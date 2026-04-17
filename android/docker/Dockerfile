# To build the image (executed from this Dockerfile directory):
# podman build . -t mullvadvpn-app-build-android
#
# To build using the image:
# podman run --rm \
#     -v $CARGO_TARGET_VOLUME_NAME:/cargo-target:Z \
#     -v $CARGO_REGISTRY_VOLUME_NAME:/root/.cargo/registry:Z \
#     -v $GRADLE_CACHE_VOLUME_NAME:/root/.gradle:Z \
#     -v /path/to/repository_root:/build:Z \
#     mullvadvpn-app-build-android ./android/build.sh --dev-build
#
# See the base image Dockerfile in the repository root (../../Dockerfile)
# for more information.

# === Base image (built from: ../../Dockerfile) ===
FROM ghcr.io/mullvad/mullvadvpn-app-build:c40d52f462

# === Metadata ===
LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app Android build container"
LABEL org.opencontainers.image.licenses=GPL-3.0-or-later

# === Define toolchain versions and paths ===

ARG SDK_VERSION=android-36 \
    BUILD_TOOLS_VERSION=36.1.0

# Command line tools and checksum from: https://developer.android.com/studio#command-line-tools-only
ARG COMMAND_LINE_TOOLS_VERSION=14742923 \
    COMMAND_LINE_TOOLS_SHA256_CHECKSUM=04453066b540409d975c676d781da1477479dde3761310f1a7eb92a1dfb15af7

# NDK and checksum from: https://github.com/android/ndk/wiki#supported-downloads
ARG NDK_VERSION_NAME=r27d \
    NDK_SHA1_CHECKSUM=22105e410cf29afcf163760cc95522b9fb981121 \
    MIN_SDK_VERSION=28 \
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

# Set Java home for the downloaded JDK
ENV JAVA_HOME=/opt/jdk-21

ARG JDK_SHA256_CHECKSUM=a2def047a73941e01a73739f92755f86b895811afb1f91243db214cff5bdac3f

# Make adb,apk signer, apk analyzer and other tools accessible
ENV PATH="\
${JAVA_HOME}/bin\
:${ANDROID_SDK_ROOT}/platform-tools\
:${ANDROID_SDK_ROOT}/build-tools/${BUILD_TOOLS_VERSION}\
:${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin\
:${PATH}"

# === Install/set up the image ===

RUN apt-get update -y && apt-get install -y \
    file \
    gpg \
    make \
    python \
    software-properties-common \
    unzip \
    ca-certificates-java \
    tidy \
    pcscd \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install fdroid server using pip instead to get a version that is using a later versions of androguard
# This and installing pip can be removed when updating to a newer version of debian
RUN pip install fdroidserver

# Install openjdk-21 (this cannot be done with apt-get as it would require the image to be updated to Debian 13)
RUN curl -sfLo /tmp/openjdk-21.tar.gz https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_linux-x64_bin.tar.gz && \
    echo "$JDK_SHA256_CHECKSUM /tmp/openjdk-21.tar.gz" | sha256sum -c && \
    tar -xf /tmp/openjdk-21.tar.gz -C /tmp/ && \
    mv /tmp/jdk-21.0.2 /opt/jdk-21 && \
    rm /tmp/openjdk-21.tar.gz

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

# Add rust targets
RUN rustup target add x86_64-linux-android i686-linux-android aarch64-linux-android armv7-linux-androideabi

# Install yubico-piv-tool (by building from source)
#
# NOTE: Yubico doesn't have a main signing key, instead individual devs
# sign release. On version bumps, verify the new tarball .sig and update
# the key file if a different developer signed the release.
# https://developers.yubico.com/Software_Projects/Software_Signing.html
ARG YUBICO_PIV_TOOL_VERSION=2.7.3
ARG YUBICO_PIV_TOOL_SHA256=fcb25c42f54298ece8b20684fb3c581ed9195a162cbc55180a4161501be93181
COPY yubico-piv-tool-signing-key.asc /tmp/yubico-piv-tool-signing-key.asc
RUN set -eux; \
    # Install build-only dependencies (some marked auto for later removal)
    apt-get update -y && \
    apt-get install -y --no-install-recommends \
        check \
        cmake \
        g++ \
        gengetopt \
        help2man \
        libpcsclite-dev \
        libssl-dev \
        pkg-config \
        zlib1g-dev \
    && \
    apt-mark auto \
        check \
        cmake \
        gengetopt \
        help2man \
        libpcsclite-dev \
        libssl-dev \
        pkg-config \
        zlib1g-dev \
    && \
    # Download source tarball and detached signature
    curl -sfLo /tmp/yubico-piv-tool.tar.gz \
        "https://developers.yubico.com/yubico-piv-tool/Releases/yubico-piv-tool-${YUBICO_PIV_TOOL_VERSION}.tar.gz" && \
    curl -sfLo /tmp/yubico-piv-tool.tar.gz.sig \
        "https://developers.yubico.com/yubico-piv-tool/Releases/yubico-piv-tool-${YUBICO_PIV_TOOL_VERSION}.tar.gz.sig" && \
    # Check checksum
    echo "$YUBICO_PIV_TOOL_SHA256 /tmp/yubico-piv-tool.tar.gz" | sha256sum -c && \
    # Verify signature
    export GNUPGHOME="$(mktemp -d)" && \
    gpg --batch --import /tmp/yubico-piv-tool-signing-key.asc && \
    gpg --batch --verify /tmp/yubico-piv-tool.tar.gz.sig /tmp/yubico-piv-tool.tar.gz && \
    rm -rf "$GNUPGHOME" && \
    # Extract and build
    tar -xzf /tmp/yubico-piv-tool.tar.gz -C /tmp && \
    mkdir /tmp/yubico-piv-tool-${YUBICO_PIV_TOOL_VERSION}/build && \
    cd /tmp/yubico-piv-tool-${YUBICO_PIV_TOOL_VERSION}/build && \
    cmake .. && \
    make -j"$(nproc)" && \
    make install && \
    ldconfig && \
    # Cleanup
    rm -rf /tmp/yubico-piv-tool* && \
    apt-get autoremove --purge -y && \
    rm -rf /var/lib/apt/lists/*

COPY pkcs11_java.cfg /usr/local/etc/pkcs11_java.cfg

WORKDIR /build
