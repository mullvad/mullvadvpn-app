# SPDX-License-Identifier: Apache-2.0
#
# Copyright © 2017-2019 WireGuard LLC. All Rights Reserved.

DESTDIR ?= $(OUT_DIR)
# Default to the workspace root if not set
CARGO_TARGET_DIR ?= ../../target
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

GOBUILDARCH := $(NDK_GO_ARCH_MAP_$(shell uname -m))
GOBUILDOS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
GOBUILDVERSION := 1.21.3
# TODO: Add checksum?
GOBUILDTARBALL := https://go.dev/dl/go$(GOBUILDVERSION).$(GOBUILDOS)-$(GOBUILDARCH).tar.gz
GOBUILDVERSION_NEEDED := go version go$(GOBUILDVERSION) $(GOBUILDOS)/$(GOBUILDARCH)


$(DESTDIR)/libwg.so:
	mkdir -p $(DESTDIR)
	# Build libmaybenot
	make --directory wireguard-go libmaybenot.a LIBDEST="$(DESTDIR)" TARGET="$(TARGET)" CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)"
	# Build wireguard-go
	go get -tags "linux android daita"
	chmod -fR +w "$(GOPATH)/pkg/mod"
	go build -tags "linux android daita" -ldflags="-X main.socketDirectory=/data/data/$(ANDROID_PACKAGE_NAME)/cache/wireguard" -v -o "$@" -buildmode c-shared -buildvcs=false
	rm -f $(DESTDIR)/libwg.h


clean:
	rm -f $(DESTDIR)/libwg.so
