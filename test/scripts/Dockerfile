ARG IMAGE=ghcr.io/mullvad/mullvadvpn-app-build:latest
FROM $IMAGE

RUN rustup target add x86_64-pc-windows-gnu

RUN apt-get update && apt-get install -y \
    mtools pkg-config libssl-dev

RUN git clone https://github.com/the-tcpdump-group/libpcap.git
RUN apt-get install -y autoconf flex bison
RUN cd libpcap \
    && ./autogen.sh \
    && ./configure --enable-remote=yes --enable-dbus=no --enable-shared=no \
    && make \
    && cp libpcap.a /usr/lib/x86_64-linux-gnu/libpcap.a
