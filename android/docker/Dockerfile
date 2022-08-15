FROM debian:10

RUN apt-get update && apt-get install -y \
    curl \
    file \
    gcc \
    git \
    gpg \
    make \
    python \
    software-properties-common \
    unzip

# Install OpenJDK 8
COPY adopt-openjdk-apt-key.pgp /tmp/adopt-openjdk-apt-key.pgp

RUN apt-key add < /tmp/adopt-openjdk-apt-key.pgp && \
    rm /tmp/adopt-openjdk-apt-key.pgp && \
    add-apt-repository -y https://adoptopenjdk.jfrog.io/adoptopenjdk/deb/ && \
    apt-get update && \
    apt-get install -y adoptopenjdk-8-hotspot

# Install Android SDK
RUN curl -sf https://dl.google.com/android/repository/sdk-tools-linux-4333796.zip -L -o /tmp/tools_r26.1.1-linux.zip && \
    cd /tmp && \
    echo "92ffee5a1d98d856634e8b71132e8a95d96c83a63fde1099be3d86df3106def9 tools_r26.1.1-linux.zip" | sha256sum -c && \
    mkdir -p /opt/android && \
    cd /opt/android && \
    unzip -q /tmp/tools_r26.1.1-linux.zip && \
    rm /tmp/tools_r26.1.1-linux.zip && \
    yes | /opt/android/tools/bin/sdkmanager "platforms;android-30" "build-tools;30.0.3" "platform-tools"

ENV ANDROID_HOME="/opt/android"

# Install Android NDK
RUN cd /tmp && \
    curl -sf -L -O https://dl.google.com/android/repository/android-ndk-r20b-linux-x86_64.zip && \
    echo "8381c440fe61fcbb01e209211ac01b519cd6adf51ab1c2281d5daad6ca4c8c8c android-ndk-r20b-linux-x86_64.zip" | sha256sum -c && \
    cd /opt/android && \
    unzip -q /tmp/android-ndk-r20b-linux-x86_64.zip && \
    rm /tmp/android-ndk-r20b-linux-x86_64.zip

ENV ANDROID_NDK_HOME="/opt/android/android-ndk-r20b" \
    NDK_TOOLCHAIN_DIR="/opt/android/android-ndk-r20b/toolchains/llvm/prebuilt/linux-x86_64/bin"

# Install golang
ENV GOLANG_VERSION 1.18.5
# Checksum from: https://go.dev/dl/
ENV GOLANG_HASH 9e5de37f9c49942c601b191ac5fba404b868bfc21d446d6960acc12283d6e5f2
COPY goruntime-boottime-over-monotonic.diff /tmp/goruntime-boottime-over-monotonic.diff
RUN cd /tmp && \
    curl -sf -L -o go.tgz https://go.dev/dl/go${GOLANG_VERSION}.linux-amd64.tar.gz && \
    echo "$GOLANG_HASH go.tgz" | sha256sum -c && \
    cd /opt && \
    tar -xzf /tmp/go.tgz && \
    patch -p1 -f -N -r- -d "/opt/go" < /tmp/goruntime-boottime-over-monotonic.diff && \
    mkdir /opt/go/go-path && \
    rm /tmp/goruntime-boottime-over-monotonic.diff /tmp/go.tgz

ENV GOROOT=/opt/go GOPATH=/opt/go/go-path PATH=${PATH}:/opt/go/bin

# Install Rust
COPY cargo-config.toml /root/.cargo/config

ENV PATH="${PATH}:/root/.cargo/bin" \
    AR_x86_64_linux_android="${NDK_TOOLCHAIN_DIR}/x86_64-linux-android-ar" \
    AR_i686_linux_android="${NDK_TOOLCHAIN_DIR}/i686-linux-android-ar" \
    AR_aarch64_linux_android="${NDK_TOOLCHAIN_DIR}/aarch64-linux-android-ar" \
    AR_armv7_linux_androideabi="${NDK_TOOLCHAIN_DIR}/arm-linux-androideabi-ar" \
    CC_x86_64_linux_android="${NDK_TOOLCHAIN_DIR}/x86_64-linux-android21-clang" \
    CC_i686_linux_android="${NDK_TOOLCHAIN_DIR}/i686-linux-android21-clang" \
    CC_aarch64_linux_android="${NDK_TOOLCHAIN_DIR}/aarch64-linux-android21-clang" \
    CC_armv7_linux_androideabi="${NDK_TOOLCHAIN_DIR}/armv7a-linux-androideabi21-clang"

RUN curl -sf -L https://sh.rustup.rs > /tmp/rustup.sh && \
    cd /tmp && \
    echo "173f4881e2de99ba9ad1acb59e65be01b2a44979d83b6ec648d0d22f8654cbce rustup.sh" | sha256sum -c && \
    chmod +x rustup.sh && \
    ./rustup.sh -y && \
    rm rustup.sh && \
    rustup target add x86_64-linux-android i686-linux-android aarch64-linux-android armv7-linux-androideabi

CMD ["./build-apk.sh", "--no-docker"]
