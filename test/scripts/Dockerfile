ARG IMAGE=ghcr.io/mullvad/mullvadvpn-app-build:latest
FROM $IMAGE

RUN rustup target add x86_64-pc-windows-gnu

RUN apt-get update && apt-get install -y \
    mtools pkg-config libssl-dev libpcap-dev
