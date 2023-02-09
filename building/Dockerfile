# To build the image (executed from the repository root):
# podman build -f building/Dockerfile . -t mullvadvpn-app-build
#
# To run the image and build the app you need to mount the app's source directory into the
# container. You also probably want to mount in a directory for CARGO_HOME, so each container
# does not need to start from scratch with cloning the crates.io index, download all
# dependencies and building everything.
#
# podman run --rm \
#     -v /path/to/container_cache/target:/root/.cargo/target:Z \
#     -v /path/to/container_cache/registry:/root/.cargo/registry:Z \
#     -v .:/build:Z \
#     mullvadvpn-app-build ./build.sh
#
# And add -e TARGETS="aarch64-unknown-linux-gnu" to build for ARM64

# Debian 10 is the oldest supported distro. It has the oldest glibc that we support
# This checksum points to a 10.13-slim image.
FROM debian@sha256:557ee531b81ce380d012d83b7bb56211572e5d6088d3e21a3caef7d7ed7f718b

LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app Linux build container"
LABEL org.opencontainers.image.licenses=GPL-3.0

# === Define toolchain versions and paths ===

ENV CARGO_TARGET_DIR=/root/.cargo/target

ENV GOLANG_VERSION=1.18.5 \
    GOLANG_HASH=9e5de37f9c49942c601b191ac5fba404b868bfc21d446d6960acc12283d6e5f2

# === Install/set up the image ===

RUN dpkg --add-architecture arm64 && apt-get update -y && apt-get install -y \
    git \
    curl \
    gcc gcc-aarch64-linux-gnu \
    libdbus-1-dev libdbus-1-dev:arm64 \
    rpm \
    protobuf-compiler \
    # For cross-compiling/linting towards Windows
    gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/*

# === Rust ===

# Install latest stable Rust toolchain for both x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu,
# plus x86_64-pc-windows-gnu for Windows cross-compilation/linting
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y \
    --default-toolchain stable \
    --profile minimal \
    --component clippy \
    --target aarch64-unknown-linux-gnu \
    --target x86_64-pc-windows-gnu

ENV PATH=/root/.cargo/bin:$PATH
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc" \
    PKG_CONFIG_SYSROOT_DIR_aarch64_unknown_linux_gnu=/usr/lib/aarch64-linux-gnu

# === Volta for npm + node ===

ENV PATH=/root/.volta/bin:$PATH
# volta seemingly does not have a way to explicitly install the toolchain
# versions from package.json, but `node --version` triggers an install
COPY gui/package.json .
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
