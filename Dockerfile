# To build the image:
# docker build . -t mullvadvpn/mullvadvpn-app-build
# To push the image to our docker hub:
# docker push mullvadvpn/mullvadvpn-app-build
FROM debian:stable@sha256:75f7d0590b45561bfa443abad0b3e0f86e2811b1fc176f786cd30eb078d1846f
RUN apt update -y
RUN apt install build-essential \
	gcc \
	libdbus-1-dev \
	rpm \
	go2 \
	binutils \
	curl \
	p7zip-full \
	git -y
RUN mkdir /mvd
CMD tail -f /dev/null
