#!/usr/bin/env bash

set -eu

# Pinned to commit hash for tag v4.24.3
PROTOBUF_COMMIT_REF=ee1355459c9ce7ffe264bc40cfdc7b7623d37e99

# Install common tools
apt-get update -y
apt-get install -y git curl jq make gcc

# Install protobuf
case ${1-} in
    linux)
        # Debian 11 ships a too old version, so build it from source
        apt-get install -y --mark-auto cmake g++
        git clone https://github.com/protocolbuffers/protobuf.git /tmp/protobuf
        cd /tmp/protobuf
        git reset --hard "$PROTOBUF_COMMIT_REF"
        git submodule update --init --recursive
        cmake . -DCMAKE_CXX_STANDARD=14
        cmake --build . -j "$(nproc)"
        cmake --install .
        cd /
        rm -rf /tmp/protobuf
        apt-get autoremove -y
    ;;
    android)
        apt-get install -y protobuf-compiler libprotobuf-dev
    ;;
    *)
        echo "Usage: $0 <linux|android>" >&2
        exit 1
esac

# Cleanup
rm -rf /var/lib/apt/lists/*
