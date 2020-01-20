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
	binutils \
	curl \
	p7zip-full \
	git -y


# Install golang
ENV GOLANG_VERSION 1.13.5
# Found on https://golang.org/dl/
ENV GOLANG_HASH 512103d7ad296467814a6e3f635631bd35574cab3369a97a323c9a585ccaa569
RUN curl -Lo go.tgz https://golang.org/dl/go${GOLANG_VERSION}.linux-amd64.tar.gz && \
  echo $(sha256sum go.tgz) && \
	echo "${GOLANG_HASH} go.tgz" | sha256sum -c - && \
	tar -C /usr/local -xzf go.tgz && \
	rm go.tgz && \
	rm -rf /var/lib/apt/lists/* && \
	export PATH="/usr/local/go/bin:$PATH" && \
	go version

ENV GOPATH /go
ENV PATH $GOPATH/bin:/usr/local/go/bin:$PATH
RUN mkdir -p "$GOPATH/src" "$GOPATH/bin" && chmod -R 777 "$GOPATH"

RUN mkdir /mvd
CMD tail -f /dev/null
