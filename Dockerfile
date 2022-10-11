# To build the image:
# docker build . -t quay.io/mullvad/mullvadvpn-app-build
# To push the image to Quay.io:
# docker push quay.io/mullvad/mullvadvpn-app-build
FROM debian:stretch@sha256:a5934d79acb9d1182ef5c747e23e462784f6345479e33b40c979fbe8dce5db40
RUN apt update -y && apt install build-essential \
	gcc \
	libdbus-1-dev \
	rpm \
	binutils \
	curl \
	p7zip-full \
	git -y && \
	rm -rf /var/lib/apt/lists/*


# Install golang
ENV GOLANG_VERSION 1.18.5
# Checksum from: https://go.dev/dl/
ENV GOLANG_HASH 9e5de37f9c49942c601b191ac5fba404b868bfc21d446d6960acc12283d6e5f2
RUN curl -Lo go.tgz https://go.dev/dl/go${GOLANG_VERSION}.linux-amd64.tar.gz && \
	echo $(sha256sum go.tgz) && \
	echo "${GOLANG_HASH} go.tgz" | sha256sum -c - && \
	tar -C /usr/local -xzf go.tgz && \
	rm go.tgz

ENV GOPATH /go
ENV PATH $GOPATH/bin:/usr/local/go/bin:$PATH
RUN go version
RUN mkdir -p "$GOPATH/src" "$GOPATH/bin" && chmod -R 777 "$GOPATH"

RUN mkdir /mvd
CMD tail -f /dev/null
