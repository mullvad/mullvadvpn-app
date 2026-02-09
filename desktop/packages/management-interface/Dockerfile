FROM debian@sha256:77f46c1cf862290e750e913defffb2828c889d291a93bdd10a7a0597720948fc

LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app management interface Linux build container"
LABEL org.opencontainers.image.licenses=GPL-3.0

RUN mkdir -p /build/bin

WORKDIR /build

COPY package.json /build
COPY ./scripts/container/build-grpc-tools-binaries.sh /build
COPY ./scripts/container/generate-bindings.sh /build

RUN chmod +x /build/build-grpc-tools-binaries.sh && \
    chmod +x /build/generate-bindings.sh 

    # Install build dependencies required for building grpc-tools (grpc-node) 
RUN apt-get update -y && \
    apt-get install -y --mark-auto build-essential cmake curl file g++ git jq && \
    rm -rf /var/lib/apt/lists/*

    # Install npm dependencies with volta
RUN curl https://get.volta.sh | bash && \
    # Source bashrc to read the paths added by volta
    . /root/.bashrc && \
    # Install node/npm
    volta install node && \
    # Install node dependencies
    npm install

    # Clone grpc-node repo
RUN git clone --recursive https://github.com/grpc/grpc-node

    # Copy our custom build script to the grpc-tools subdirectory
RUN mv /build/build-grpc-tools-binaries.sh /build/grpc-node/packages/grpc-tools

    # Checkout a pre-defined version of the repo
RUN cd /build/grpc-node/packages/grpc-tools && git checkout ccd29b27d28ce8937f8250f72e5e6027ed5af09a && \
    # Build the grpc-tools binaries used to generate JS bindings
    ./build-grpc-tools-binaries.sh

    # Clean up
 RUN rm /build/package.json && \
    # Trim image size by removing dependencies
    apt-get autoremove -y


