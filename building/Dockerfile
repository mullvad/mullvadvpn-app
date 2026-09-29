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
#     -e CARGO_TARGET_DIR=/cargo-target/target \
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
LABEL org.opencontainers.image.licenses=GPL-3.0-or-later

# === Common setup ===

# Install packages from a fixed point in time on snapshot.debian.org, so that every build of
# this image installs the exact same package versions. This helps make the image reproducible.
# Bump the timestamp to pick up newer packages. snapshot.debian.org resolves it to the latest
# snapshot taken at or before that time, separately for each archive.
# The current timestamp is after the final Debian 11 LTS security update.
ARG DEBIAN_SNAPSHOT=20260901T000000Z
COPY building/setup-debian-snapshot.sh /tmp/
RUN /tmp/setup-debian-snapshot.sh && rm /tmp/setup-debian-snapshot.sh

COPY building/install-base-tools.sh /tmp/
RUN /tmp/install-base-tools.sh && rm /tmp/install-base-tools.sh

COPY building/install-rust.sh rust-toolchain.toml /tmp/rust/
RUN /tmp/rust/install-rust.sh linux && rm -r /tmp/rust
ENV PATH=/root/.cargo/bin:$PATH

# === Install/set up the image ===

RUN dpkg --add-architecture arm64 && apt-get update -y && apt-get install -y \
    gcc-aarch64-linux-gnu \
    # App dependencies
    libdbus-1-dev libdbus-1-dev:arm64 \
    rpm \
    # For cross-compiling/linting towards Windows
    gcc-mingw-w64-x86-64 \
    # For building Arch Linux packages
    libarchive-tools \
    && rm -rf /var/lib/apt/lists/*

# Rust cross-compilation for: aarch64
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc" \
    PKG_CONFIG_SYSROOT_DIR_aarch64_unknown_linux_gnu=/usr/lib/aarch64-linux-gnu

# === Volta for npm + node ===

ENV PATH=/root/.volta/bin:$PATH

# Pin and verify volta version. volta publishes no checksums, so compute it from ${VOLTA_URL}
ARG VOLTA_VERSION=2.0.2
ARG VOLTA_SHA256=6cec054c911fb925b629a09455775af6e95dc0f5694a4c28b63979ab9ef18037

ARG VOLTA_URL=https://github.com/volta-cli/volta/releases/download/v$VOLTA_VERSION/volta-$VOLTA_VERSION-linux.tar.gz

# volta seemingly does not have a way to explicitly install the toolchain
# versions from package.json, but `node --version` triggers an install
COPY desktop/package.json .
RUN curl --proto '=https' --tlsv1.2 -sSfL -o volta.tar.gz "$VOLTA_URL" && \
    echo "$VOLTA_SHA256  volta.tar.gz" | sha256sum -c - && \
    mkdir -p /root/.volta/bin && \
    tar -xzf volta.tar.gz -C /root/.volta/bin && \
    rm volta.tar.gz && \
    volta setup && \
    node --version && rm package.json

WORKDIR /build
