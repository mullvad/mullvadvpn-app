ARG IMAGE=ghcr.io/mullvad/mullvadvpn-app-build:latest
FROM $IMAGE

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpcap-dev
