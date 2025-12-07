# To build the image (executed from the repository root):
# podman build -f building/Dockerfile . -t mullvadvpn-app-build
#
# To run the image and build the app you need to mount the app's source directory into the
# container. You also probably want to mount in a directory for CARGO_HOME, so each container
# does not need to start from scratch with cloning the crates.io index, download all
# dependencies and building everything.
#
# podman run --rm \
#     -v $CARGO_TARGET_VOLUME_NAME:/cargo-target:Z \
#     -v $CARGO_REGISTRY_VOLUME_NAME:/root/.cargo/registry:Z \
#     -v /path/to/repository_root:/build:Z \
#     mullvadvpn-app-build ./build.sh
#
# And add -e TARGETS="aarch64-unknown-linux-gnu" to build for ARM64

# When building the app, we must link towards the oldest glibc version we want to support.
# This is currently glibc 2.31 which is the version in Debian 11.
# When updating this base image, to find the checksum, run: podman inspect <image> | jq '.[]["Digest"]'
# This checksum points to a 11.6-slim image.
FROM debian@sha256:77f46c1cf862290e750e913defffb2828c889d291a93bdd10a7a0597720948fc

LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app Linux build container"
LABEL org.opencontainers.image.licenses=GPL-3.0

# === Define toolchain versions and paths ===

ENV CARGO_TARGET_DIR=/cargo-target/target

ARG GOLANG_VERSION=1.21.3 \
    GOLANG_HASH=1241381b2843fae5a9707eec1f8fb2ef94d827990582c7c7c32f5bdfbfd420c8

# Pinned to commit hash for tag v4.24.3
ARG PROTOBUF_COMMIT_REF=ee1355459c9ce7ffe264bc40cfdc7b7623d37e99

# === Install/set up the image ===

RUN dpkg --add-architecture arm64 && apt-get update -y && apt-get install -y \
    git \
    curl \
    make \
    gcc gcc-aarch64-linux-gnu \
    rpm \
    # For cross-compiling/linting towards Windows
    gcc-mingw-w64-x86-64 \
    # For building Arch Linux packages
    libarchive-tools \
    && rm -rf /var/lib/apt/lists/*

# === Rust ===

ARG RUST_VERSION=stable

# Install the Rust toolchain for both x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu,
# plus x86_64-pc-windows-gnu for Windows cross-compilation/linting
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y \
    --default-toolchain $RUST_VERSION \
    --profile minimal \
    --component clippy \
    --target aarch64-unknown-linux-gnu \
    --target x86_64-pc-windows-gnu

ENV PATH=/root/.cargo/bin:$PATH
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc" \
    PKG_CONFIG_SYSROOT_DIR_aarch64_unknown_linux_gnu=/usr/lib/aarch64-linux-gnu

# === protobuf (for compiling .proto files) ===

RUN apt-get update -y && \
    apt-get install -y --mark-auto cmake g++ && \
    rm -rf /var/lib/apt/lists/* && \
    git clone https://github.com/protocolbuffers/protobuf.git && \
    cd protobuf && \
    git reset --hard "$PROTOBUF_COMMIT_REF" && \
    git submodule update --init --recursive && \
    cmake . -DCMAKE_CXX_STANDARD=14 && \
    cmake --build . -j $(nproc) && \
    cmake --install . && \
    cd .. && rm -rf protobuf && \
    apt-get autoremove -y

# === Volta for npm + node ===

ENV PATH=/root/.volta/bin:$PATH
# volta seemingly does not have a way to explicitly install the toolchain
# versions from package.json, but `node --version` triggers an install
COPY desktop/package.json .
RUN curl https://get.volta.sh | bash && node --version && rm package.json

# === Golang ===

# Install golang
# Checksum from: https://go.dev/dl/
RUN curl -Lo go.tgz https://go.dev/dl/go${GOLANG_VERSION}.linux-amd64.tar.gz && \
    echo "${GOLANG_HASH} go.tgz" | sha256sum -c - && \
    tar -C /usr/local -xzf go.tgz && \
    rm go.tgz
ENV PATH=/usr/local/go/bin:$PATH


WORKDIR /build
