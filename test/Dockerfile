ARG IMAGE=ghcr.io/mullvad/mullvadvpn-app-build:latest
FROM $IMAGE

RUN apt update -y && \
    apt install -y mtools  && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpcap-dev
