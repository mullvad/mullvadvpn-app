# This image builds the grpc-tools binaries from source, which are then
# used to generate JS/TS bindings for our .proto files.
#
# Refer to the README.md in <root>/desktop/packages/management-interface
# for more information on how to use this image.

# This checksum points to a Debian 11.6-slim image.
# Note that this image is independent, and as such does not need to be
# kept in sync with any other image in the mullvadvpn-app repo.
FROM debian@sha256:77f46c1cf862290e750e913defffb2828c889d291a93bdd10a7a0597720948fc

LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app management interface Linux build container"
LABEL org.opencontainers.image.licenses=GPL-3.0

# Pinned to commit hash for tag 1.14.3
ARG GRPC_NODE_REF=ccd29b27d28ce8937f8250f72e5e6027ed5af09a

# === Setup folders ===

RUN mkdir -p \
    # grpc-tools binaries are built in this folder
    /grpc-tools \
    # grpc-tools binaries are output to this folder and serves as the
    /grpc-tools-binaries \
    # Scripts to make it easier to run things within the container
    # are copied to this folder
    /scripts

# === Setup volumes ===

# input folder for proto files to generate bindings for
VOLUME /proto
# output folder for bindings generated for the proto files
VOLUME /proto-bindings

COPY package.json /grpc-tools-binaries
COPY package-lock.json /grpc-tools-binaries

COPY ./scripts/container/generate-bindings.sh /scripts
RUN chmod +x /scripts/generate-bindings.sh

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
    cd /grpc-tools-binaries && npm ci

# === Clone grpc-node repository ===

RUN cd /grpc-tools/ && \
    git clone --recursive https://github.com/grpc/grpc-node && \
    # Checkout a pre-defined version of the repo
    cd grpc-node && git reset --hard "$GRPC_NODE_REF"

# === Build grpc-tools (grpc-node) ===

    # Source bashrc to read the paths added by volta
RUN . ~/.bashrc && \
    cd /grpc-tools/grpc-node/packages/grpc-tools && \
    # Compile the protoc/grpc_node binaries and copy them to the
    # binaries folder.
    #
    # This cmake command is based on the build script in the grpc-tools
    # package (grpc-node/packages/grpc-tools/build_binaries.sh), but is
    # stripped down to only build for the current platform/arch.
    cmake . && cmake --build . --target clean && cmake --build . -- -j 12 && \
    # Note: protoc is a symlink to a versioned file, hence why we need the -L flag.
    cp -L "deps/protobuf/protoc" "/grpc-tools-binaries/protoc" && \
    cp "grpc_node_plugin" "/grpc-tools-binaries/grpc_node_plugin" && \
    # 2. Gather all google protobuf files in the bin folder in grpc-tools
    # and then copy them to the node_modules/.bin folder
    node "copy_well_known_protos.js" && \
    cp -r "bin/google" "/grpc-tools-binaries/google" && \
    # 3. Copy the "bin" files defined in grpc-tools package.json
    # (grpc-node/packages/grpc-tools/package.json) to the node_modules/.bin folder.
    #
    # TODO: We could use jq to read these "bin" files from the package.json
    cp "bin/protoc.js" "/grpc-tools-binaries/grpc_tools_node_protoc" && \
    cp "bin/protoc_plugin.js" "/grpc-tools-binaries/grpc_tools_node_protoc_plugin"

# === Remove build dependencies to trim image size ===

RUN apt-get autoremove -y

# === Set the entrypoint to automatically generate bindings

# Note that this will not work if the associated volumes are not included
ENTRYPOINT ["/bin/bash", "-c", ". ~/.bashrc && /scripts/generate-bindings.sh"]
