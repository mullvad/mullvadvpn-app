# mullvadvpn-app-build-node-grpc-bindings
#
# This image builds the grpc-tools binaries from source, which are then
# used to generate node gRPC bindings from .proto files.
#
# Refer to README.md for the management-interface package for more
# information on how to build and use this image.

# This checksum points to a Debian 11.6-slim image.
# Note that this image is independent, and as such does not need to be
# kept in sync with any other image in the mullvadvpn-app repo.
FROM debian@sha256:77f46c1cf862290e750e913defffb2828c889d291a93bdd10a7a0597720948fc

LABEL org.opencontainers.image.source=https://github.com/mullvad/mullvadvpn-app
LABEL org.opencontainers.image.description="Mullvad VPN app management interface Linux build container"
LABEL org.opencontainers.image.licenses=GPL-3.0

# Pinned to grpc-node repository's commit hash for tag 1.14.3
ARG GRPC_NODE_REF=ccd29b27d28ce8937f8250f72e5e6027ed5af09a

# === Setup volumes ===

# Input folder for .proto files to generate bindings for
VOLUME /proto
# Output folder for bindings generated for the .proto files
VOLUME /proto-bindings

# === Setup work folders ===

RUN mkdir -p \
    # The grpc-node folder is where the grpc-node repo will be cloned to.
    /node-grpc-bindings/grpc-node \
    # The bindings-dependencies folder holds dependencies needed to
    # node gRPC generate bindings from .proto files.
    #
    # The folder contains:
    # - The grpc-tools binaries built from the grpc-node repo.
    # - Additional npm dependencies needed to generate bindings.
    /node-grpc-bindings/bindings-dependencies \
    # The entrypoint folder contains the container entrypoint script
    # which invokes the generation of node gRPC bindings for the
    # .proto files in the /proto volume.
    /node-grpc-bindings/entrypoint

# === Copy entrypoint script ===

COPY packages/management-interface/scripts/container-entrypoint-generate-bindings.sh /node-grpc-bindings/entrypoint
RUN chmod +x /node-grpc-bindings/entrypoint/container-entrypoint-generate-bindings.sh

# === Copy node config ===

COPY ../../package-lock.json /node-grpc-bindings/bindings-dependencies
COPY packages/management-interface/package.json /node-grpc-bindings/bindings-dependencies

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
    cd /node-grpc-bindings/bindings-dependencies && npm ci

# === Clone grpc-node repository ===

RUN cd /node-grpc-bindings/ && \
    git clone --recursive https://github.com/grpc/grpc-node && \
    # Checkout a pre-defined version of the repo
    cd grpc-node && git reset --hard "$GRPC_NODE_REF"

# === Build grpc-tools (grpc-node) ===

    # Source bashrc to read the paths added by volta
RUN . ~/.bashrc && \
    cd /node-grpc-bindings/grpc-node/packages/grpc-tools && \
    # Compile the protoc/grpc_node binaries and copy them to the
    # node-bindings-dependencies folder.
    #
    # This cmake command is based on the build script in the grpc-tools
    # package (grpc-node/packages/node-grpc-bindings/build_binaries.sh), but is
    # stripped down to only build for the current platform/arch.
    cmake . && cmake --build . --target clean && cmake --build . -- -j 12 && \
    # Note: protoc is a symlink to a versioned file, hence why we need the -L flag.
    cp -L "deps/protobuf/protoc" "/node-grpc-bindings/bindings-dependencies/protoc" && \
    cp "grpc_node_plugin" "/node-grpc-bindings/bindings-dependencies/grpc_node_plugin" && \
    # 2. Gather all google protobuf files in the bin folder in the
    # grpc-tools package.
    node "copy_well_known_protos.js" && \
    cp -r "bin/google" "/node-grpc-bindings/bindings-dependencies/google" && \
    # 3. Copy the "bin" files defined in grpc-tools package.json
    # (grpc-node/packages/node-grpc-bindings/package.json)
    #
    # TODO: We could use jq to read these "bin" files from the package.json
    cp "bin/protoc.js" "/node-grpc-bindings/bindings-dependencies/grpc_tools_node_protoc" && \
    cp "bin/protoc_plugin.js" "/node-grpc-bindings/bindings-dependencies/grpc_tools_node_protoc_plugin"

# === Remove build dependencies to trim image size ===

RUN apt-get autoremove -y

# === Set the entrypoint to run script to generate bindings

# Note that this command will not work if the associated volumes are
# not attached when the container is run.
ENTRYPOINT ["/bin/bash", "-c", ". ~/.bashrc && /node-grpc-bindings/entrypoint/container-entrypoint-generate-bindings.sh"]
