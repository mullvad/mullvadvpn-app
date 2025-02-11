# SPDX-License-Identifier: Apache-2.0
#
# Copyright © 2017-2019 WireGuard LLC. All Rights Reserved.

DESTDIR ?= $(OUT_DIR)
# Default to the workspace root if not set
CARGO_TARGET_DIR ?= $(CURDIR)/../../target
TARGET ?=

NDK_GO_ARCH_MAP_x86 := 386
NDK_GO_ARCH_MAP_x86_64 := amd64
NDK_GO_ARCH_MAP_arm := arm
NDK_GO_ARCH_MAP_arm64 := arm64
NDK_GO_ARCH_MAP_mips := mipsx
NDK_GO_ARCH_MAP_mips64 := mips64x

export CGO_CFLAGS := $(CFLAGS)
export CGO_LDFLAGS := $(LDFLAGS)
export CC := $(ANDROID_C_COMPILER)
export GOARCH := $(NDK_GO_ARCH_MAP_$(ANDROID_ARCH_NAME))
export GOOS := android
export CGO_ENABLED := 1

default: $(DESTDIR)/libwg.so

$(DESTDIR)/libwg.so:
	mkdir -p $(DESTDIR)
	# Build libmaybenot
	make --directory wireguard-go/maybenot-ffi $(DESTDIR)/libmaybenot.a TARGET="$(TARGET)" CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)"
	# Build wireguard-go
	go get -tags "linux android daita"
	chmod -fR +w "$(GOPATH)/pkg/mod"
	# The `-buildid=` and `-trimpath` flags are needed to make the build reproducible.
	go build -tags "linux android daita" -ldflags="-buildid= -X main.socketDirectory=/data/data/$(ANDROID_PACKAGE_NAME)/cache/wireguard" -v -o "$@" -buildmode c-shared -buildvcs=false -trimpath
	rm -f $(DESTDIR)/libwg.h

clean:
	rm -f $(DESTDIR)/libwg.so
