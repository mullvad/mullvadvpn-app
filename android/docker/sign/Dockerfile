# To build the image (executed from this Dockerfile directory):
# podman build . -t mullvadvpn-app-sign-android
#

# === Base image (should be latest stable debian) ===
FROM debian@sha256:90e43532b8275215005dd071c9e4495baaa4e43b937f8519c7dedc2d35a60b45

# === Metadata ===
LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app Android sign container"
LABEL org.opencontainers.image.licenses=GPL-3.0-or-later

# === Define toolchain versions and paths ===

ARG SDK_VERSION=platforms;android-36 \
    BUILD_TOOLS_VERSION=36.0.0

# Command line tools and checksum from: https://developer.android.com/studio#command-line-tools-only
ARG COMMAND_LINE_TOOLS_VERSION=13114758 \
    COMMAND_LINE_TOOLS_SHA256_CHECKSUM=7ec965280a073311c339e571cd5de778b9975026cfcbe79f2b1cdcb1e15317ee

ENV ANDROID_SDK_ROOT=/opt/android
# ANDROID_HOME is kept for backwards compatibility
ENV ANDROID_HOME=$ANDROID_SDK_ROOT

# Set Java home for the downloaded JDK
ENV JAVA_HOME=/opt/jdk-21

# Make apksigner accessible
ENV PATH="${JAVA_HOME}/bin:${ANDROID_SDK_ROOT}/build-tools/${BUILD_TOOLS_VERSION}:${PATH}"

# === Install/set up the image ===

RUN apt-get update -y && apt-get install -y \
    curl \
    unzip \
    ykcs11 \
    && rm -rf /var/lib/apt/lists/*

# Install openjdk-21 (this does not work with apt-get for some reason)
RUN curl -sfLo /tmp/openjdk-21.tar.gz https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_linux-x64_bin.tar.gz && \
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
RUN yes | $ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager $SDK_VERSION "build-tools;$BUILD_TOOLS_VERSION"

# Create config file
RUN printf '%s\n' \
"name = YKCS11" \
"description = SunPKCS11 via YKCS11" \
"library = /usr/lib/x86_64-linux-gnu/libykcs11.so.2" \
"slotListIndex = 1" > ~/ykcs11.conf

WORKDIR /build
