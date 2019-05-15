# To build the image:
# docker build . -t mullvadvpn/mullvadvpn-app-build
# To push the image to our docker hub:
# docker push mullvadvpn/mullvadvpn-app-build
FROM debian:unstable
RUN apt update -y
RUN apt install build-essential \
	gcc \
	libdbus-1-dev \
	rpm \
	gconf2 \
	go2 \
	binutils \
	curl \
	p7zip-full \
	git -y
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --default-toolchain none
ENV PATH="/root/.cargo/bin/:${PATH}"
RUN mkdir /mvd
CMD tail -f /dev/null
