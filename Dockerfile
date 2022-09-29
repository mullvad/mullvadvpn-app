# To build the image:
# podman build . -t mullvadvpn-app-build
#
# To run the image and build the app you need to mount the app's source directory into the
# container. You also probably want to mount in a directory for CARGO_HOME, so each container
# does not need to start from scratch with cloning the crates.io index, download all
# dependencies and building everything.
#
# podman run --rm \
#     -v ~/.cargo:/root/.cargo:Z \
#     -v .:/build:Z \
#     mullvadvpn-app-build ./build.sh

# Debian 10 is the oldest supported distro. It has the oldest glibc that we support
FROM debian:10.13-slim@sha256:557ee531b81ce380d012d83b7bb56211572e5d6088d3e21a3caef7d7ed7f718b

# === Define toolchain versions and paths ===

ENV CARGO_HOME=/root/.cargo
ENV CARGO_TARGET_DIR=/root/.cargo/target

ENV GOLANG_VERSION 1.18.5
ENV GOLANG_HASH 9e5de37f9c49942c601b191ac5fba404b868bfc21d446d6960acc12283d6e5f2

# === Install/set up the image ===

RUN apt-get update -y && apt-get install -y \
    git \
    curl \
    gcc \
    libdbus-1-dev \
    rpm \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install latest stable Rust toolchain
ENV PATH "/root/.cargo/bin:$PATH"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- --default-toolchain stable --profile minimal -y

ENV PATH /root/.volta/bin:$PATH
# volta seemingly does not have a way to explicitly install the toolchain
# versions from package.json, but `node --version` triggers an install
COPY gui/package.json .
RUN curl https://get.volta.sh | bash && node --version && rm package.json

# Install golang
# Checksum from: https://go.dev/dl/
RUN curl -Lo go.tgz https://go.dev/dl/go${GOLANG_VERSION}.linux-amd64.tar.gz && \
    echo "${GOLANG_HASH} go.tgz" | sha256sum -c - && \
    tar -C /usr/local -xzf go.tgz && \
    rm go.tgz
ENV PATH /usr/local/go/bin:$PATH

WORKDIR /build
