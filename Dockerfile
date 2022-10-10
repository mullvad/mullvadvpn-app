# To build the image:
# podman build . -t mullvadvpn-app-build
#
# To run the image and build the app you need to mount the app's source directory into the
# container. You also probably want to mount in a directory for CARGO_HOME and CARGO_TARGET_DIR
# so each container does not need to start from scratch with cloning the crates.io index,
# download all dependencies and building everything.
#
# podman run --rm \
#     -v ~/.cargo:/root/.cargo:Z -e=CARGO_TARGET_DIR=/root/.cargo/target \
#     -v .:/build:Z -w /build \
#     -it mullvadvpn-app-build

# Debian 10 is the oldest supported distro. It has the oldest glibc that we support
FROM debian:10@sha256:604db908f7ce93379b1289c0c7ba73b252002087a3fa64fe904b430083ba5f69

RUN apt-get update -y && apt-get install -y \
	git \
	curl \
	gcc \
	libdbus-1-dev \
	rpm \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install latest stable Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- --default-toolchain stable --profile minimal -y

# Ugly way to make volta install our desired nodejs toolchain version
COPY gui/package.json .
RUN curl https://get.volta.sh | bash && bash -c 'source ~/.bashrc && node --version' && rm package.json

# Install golang
ENV GOLANG_VERSION 1.18.5
# Checksum from: https://go.dev/dl/
ENV GOLANG_HASH 9e5de37f9c49942c601b191ac5fba404b868bfc21d446d6960acc12283d6e5f2
RUN curl -Lo go.tgz https://go.dev/dl/go${GOLANG_VERSION}.linux-amd64.tar.gz && \
	echo "${GOLANG_HASH} go.tgz" | sha256sum -c - && \
	tar -C /usr/local -xzf go.tgz && \
	rm go.tgz
ENV PATH /usr/local/go/bin:$PATH

