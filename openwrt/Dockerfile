# To build the image (executed from the repository root):
# podman build -f openwrt/Dockerfile . -t mullvadvpn-app-openwrt
FROM ghcr.io/openwrt/rootfs:x86_64-openwrt-24.10
RUN mkdir -p /var/lock/
#RUN opkg update
# Install actual dependencies that the `mullvad-cli` installer (currently) needs.
#RUN opkg install bash
# These directories are required by the mullvad-daemon.
# We should revise this. Among other things, we probably don't want to log to disk
# on a router.
RUN mkdir -p /var/cache
RUN mkdir -p /var/log
RUN mkdir -p /var/run
COPY mullvad-daemon .
COPY mullvad .
