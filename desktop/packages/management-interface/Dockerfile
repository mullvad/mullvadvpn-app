FROM debian@sha256:77f46c1cf862290e750e913defffb2828c889d291a93bdd10a7a0597720948fc

LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app management interface Linux build container"
LABEL org.opencontainers.image.licenses=GPL-3.0

# Pinned to commit hash for tag 1.14.3
ARG GRPC_NODE_REF=ccd29b27d28ce8937f8250f72e5e6027ed5af09a

# === Setup build folder ===

RUN mkdir -p /build

WORKDIR /build

COPY package.json /build
COPY ./scripts/container/build-grpc-tools-binaries.sh /build
COPY ./scripts/container/generate-bindings.sh /build

RUN chmod +x /build/build-grpc-tools-binaries.sh && \
    chmod +x /build/generate-bindings.sh

# === Install build dependencies ===

RUN apt-get update -y && \
    apt-get install -y --mark-auto build-essential cmake curl file g++ git jq && \
    rm -rf /var/lib/apt/lists/*

# === Install node and npm dependencies using volta ===

    # Install volta
RUN curl https://get.volta.sh | bash && \
    # Source bashrc to read the paths added by volta
    . ~/.bashrc && \
    # Install node/npm
    volta install node && \
    # Install node dependencies
    npm install

# === Clone grpc-node repository ===

RUN git clone --recursive https://github.com/grpc/grpc-node && \
    # Checkout a pre-defined version of the repo
    cd /build/grpc-node && git reset --hard "$GRPC_NODE_REF"

# === Build grpc-tools (grpc-node) ===

    # Source bashrc to read the paths added by volta
RUN . ~/.bashrc && \
    cd /build/grpc-node/packages/grpc-tools && \
    # Build the grpc-tools binaries used to generate JS bindings
    bash /build/build-grpc-tools-binaries.sh

# === Remove build dependencies to trim image size ===

RUN apt-get autoremove -y
