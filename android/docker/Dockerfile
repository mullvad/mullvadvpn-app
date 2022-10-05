# To build the image:
# podman build . -t mullvad-android-app-build
#
# See the Dockerfile for mullvadvpn-app-build (in the repository root) for more information.

FROM mullvadvpn-app-build@sha256:<TODO>

# === Define toolchain versions and paths ===

ENV SDK_VERSION=platforms;android-33 \
    BUILD_TOOLS_VERSION=build-tools;33.0.0

ENV SDK_TOOLS_VERSION=4333796 \
    SDK_TOOLS_HASH=92ffee5a1d98d856634e8b71132e8a95d96c83a63fde1099be3d86df3106def9

ENV NDK_VERSION=r20b \
    NDK_HASH=8381c440fe61fcbb01e209211ac01b519cd6adf51ab1c2281d5daad6ca4c8c8c

ENV GRADLE_USER_HOME=/root/.gradle

ENV GOROOT=/usr/local/go \
    GOPATH=/usr/local/go/go-path

ENV ANDROID_HOME=/opt/android
ENV ANDROID_NDK_HOME=${ANDROID_HOME}/android-ndk-${NDK_VERSION}
ENV NDK_TOOLCHAIN_DIR=${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin

ENV AR_x86_64_linux_android=${NDK_TOOLCHAIN_DIR}/x86_64-linux-android-ar \
    AR_i686_linux_android=${NDK_TOOLCHAIN_DIR}/i686-linux-android-ar \
    AR_aarch64_linux_android=${NDK_TOOLCHAIN_DIR}/aarch64-linux-android-ar \
    AR_armv7_linux_androideabi=${NDK_TOOLCHAIN_DIR}/arm-linux-androideabi-ar \
    CC_x86_64_linux_android=${NDK_TOOLCHAIN_DIR}/x86_64-linux-android21-clang \
    CC_i686_linux_android=${NDK_TOOLCHAIN_DIR}/i686-linux-android21-clang \
    CC_aarch64_linux_android=${NDK_TOOLCHAIN_DIR}/aarch64-linux-android21-clang \
    CC_armv7_linux_androideabi=${NDK_TOOLCHAIN_DIR}/armv7a-linux-androideabi21-clang

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
    rm /tmp/adopt-openjdk-apt-key.pgp && \
    add-apt-repository -y https://adoptopenjdk.jfrog.io/adoptopenjdk/deb/ && \
    apt-get update && \
    apt-get install -y adoptopenjdk-8-hotspot \
    && rm -rf /var/lib/apt/lists/*

# Install base Android SDK tools
RUN curl -sfLo /tmp/sdk-tools.zip https://dl.google.com/android/repository/sdk-tools-linux-${SDK_TOOLS_VERSION}.zip && \
    echo "$SDK_TOOLS_HASH /tmp/sdk-tools.zip" | sha256sum -c && \
    mkdir -p $ANDROID_HOME && \
    unzip -q /tmp/sdk-tools.zip -d $ANDROID_HOME && \
    rm /tmp/sdk-tools.zip

# Install Android SDK
RUN yes | $ANDROID_HOME/tools/bin/sdkmanager $SDK_VERSION $BUILD_TOOLS_VERSION "platform-tools"

# Install Android NDK
RUN curl -sfLo /tmp/ndk.zip https://dl.google.com/android/repository/android-ndk-${NDK_VERSION}-linux-x86_64.zip && \
    echo "$NDK_HASH /tmp/ndk.zip" | sha256sum -c && \
    unzip -q /tmp/ndk.zip -d $ANDROID_HOME && \
    rm /tmp/ndk.zip

# Apply go patch required for Android
COPY goruntime-boottime-over-monotonic.diff /tmp/goruntime-boottime-over-monotonic.diff
RUN patch -p1 -f -N -r- -d $GOROOT < /tmp/goruntime-boottime-over-monotonic.diff && \
    rm /tmp/goruntime-boottime-over-monotonic.diff

# Add rust targets
COPY cargo-config.toml $CARGO_HOME/config
RUN rustup target add x86_64-linux-android i686-linux-android aarch64-linux-android armv7-linux-androideabi

WORKDIR /build
