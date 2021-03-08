# To build the image:
# docker build . -t quay.io/mullvad/mullvad-android-app-build
# To push the image to Quay.io:
# docker push quay.io/mullvad/mullvad-android-app-build

FROM debian@sha256:75f7d0590b45561bfa443abad0b3e0f86e2811b1fc176f786cd30eb078d1846f

RUN apt-get update -y && apt-get install -y \
    curl \
    file \
    gcc \
    git \
    make \
    python \
    unzip

# Install Android NDK
RUN cd /tmp && \
    curl -sf -L -O https://dl.google.com/android/repository/android-ndk-r20-linux-x86_64.zip && \
    test $(sha256sum android-ndk-r20-linux-x86_64.zip | cut -f1 -d' ') = "57435158f109162f41f2f43d5563d2164e4d5d0364783a9a6fab3ef12cb06ce0" && \
    mkdir /opt/android && \
    cd /opt/android && \
    unzip -q /tmp/android-ndk-r20-linux-x86_64.zip && \
    rm /tmp/android-ndk-r20-linux-x86_64.zip


ENV ANDROID_NDK_HOME="/opt/android/android-ndk-r20"


# Install Go-lang and patch it to use the appropriate monotonic clock
COPY goruntime-boottime-over-monotonic.diff /opt/goruntime-boottime-over-monotonic.diff
RUN cd /tmp && \
    curl -sf -L -O https://dl.google.com/go/go1.13.3.linux-amd64.tar.gz && \
    echo "0804bf02020dceaa8a7d7275ee79f7a142f1996bfd0c39216ccb405f93f994c0 go1.13.3.linux-amd64.tar.gz" | sha256sum --check && \
    cd /opt && \
    tar -xzf /tmp/go1.13.3.linux-amd64.tar.gz && \
    rm /tmp/go1.13.3.linux-amd64.tar.gz && \
    patch -p1 -f -N -r- -d "/opt/go" < /opt/goruntime-boottime-over-monotonic.diff

ENV PATH=${PATH}:/opt/go/bin
ENV GOROOT=/opt/go
ENV GOPATH=/opt/go-path

RUN apt-get remove -y curl && \
    apt-get autoremove -y

ENTRYPOINT []
