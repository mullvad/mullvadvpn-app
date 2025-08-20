# To build the image (executed from the repository root):
# podman build -f openwrt/Dockerfile . -t mullvadvpn-app-openwrt
FROM ghcr.io/openwrt/rootfs:x86_64-openwrt-24.10
COPY dist/mullvad-vpn-cli_2024.6-beta1-dev-8eec3d_amd64_x86_64.ipk .
RUN mkdir /var/lock/ && opkg update
# Install actuall dependencies that the `mullvad-cli` installer (currently) needs.
RUN opkg install bash
