FROM debian:stable

RUN apt-get update && apt-get install -y \
    gcc curl libdbus-1-dev protobuf-compiler
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
