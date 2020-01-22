# SPDX-License-Identifier: Apache-2.0
#
# Copyright © 2017-2019 WireGuard LLC. All Rights Reserved.

BUILDDIR ?= $(CURDIR)/build
DESTDIR ?= $(CURDIR)/../android/build/extraJni/$(ANDROID_ABI)

NDK_GO_ARCH_MAP_x86 := 386
NDK_GO_ARCH_MAP_x86_64 := amd64
NDK_GO_ARCH_MAP_arm := arm
NDK_GO_ARCH_MAP_arm64 := arm64
NDK_GO_ARCH_MAP_mips := mipsx
NDK_GO_ARCH_MAP_mips64 := mips64x

CLANG_FLAGS := --target=$(ANDROID_LLVM_TRIPLE) --gcc-toolchain=$(ANDROID_TOOLCHAIN_ROOT) --sysroot=$(ANDROID_SYSROOT)
export CGO_CFLAGS := $(CLANG_FLAGS) $(CFLAGS)
export CGO_LDFLAGS := $(CLANG_FLAGS) $(LDFLAGS)
export CC := $(ANDROID_C_COMPILER)
export GOARCH := $(NDK_GO_ARCH_MAP_$(ANDROID_ARCH_NAME))
export GOOS := android
export CGO_ENABLED := 1

default: $(DESTDIR)/libwg.so

GOBUILDARCH := $(NDK_GO_ARCH_MAP_$(shell uname -m))
GOBUILDOS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
GOBUILDVERSION := 1.12
GOBUILDTARBALL := https://dl.google.com/go/go$(GOBUILDVERSION).$(GOBUILDOS)-$(GOBUILDARCH).tar.gz
GOBUILDVERSION_NEEDED := go version go$(GOBUILDVERSION) $(GOBUILDOS)/$(GOBUILDARCH)
export GOROOT := $(BUILDDIR)/goroot
export GOPATH := $(BUILDDIR)/gopath
export PATH := $(GOROOT)/bin:$(PATH)
GOBUILDVERSION_CURRENT := $(shell $(GOROOT)/bin/go version 2>/dev/null)
ifneq ($(GOBUILDVERSION_NEEDED),$(GOBUILDVERSION_CURRENT))
$(shell rm -f $(GOROOT)/bin/go)
endif
$(GOROOT)/bin/go:
	rm -rf "$(GOROOT)"
	mkdir -p "$(GOROOT)"
	curl "$(GOBUILDTARBALL)" | tar -C "$(GOROOT)" --strip-components=1 -xzf - || { rm -rf "$(GOROOT)"; exit 1; }
	patch -p1 -f -N -r- -d "$(GOROOT)" < goruntime-boottime-over-monotonic.diff || { rm -rf "$(GOROOT)"; exit 1; }

$(shell test "$$(cat $(BUILDDIR)/.gobuildversion 2>/dev/null)" = "$(GOBUILDVERSION_CURRENT)" || rm -f "$(DESTDIR)/libwg.so")

$(DESTDIR)/libwg.so: $(GOROOT)/bin/go
	mkdir -p $(DESTDIR)
	go get -tags "linux android" || { chmod -fR +w "$(GOPATH)/pkg/mod"; rm -rf "$(GOPATH)/pkg/mod"; exit 1; }
	chmod -fR +w "$(GOPATH)/pkg/mod"
	go build -tags "linux android" -ldflags="-X main.socketDirectory=/data/data/$(ANDROID_PACKAGE_NAME)/cache/wireguard" -v -o "$@" -buildmode c-shared
	go version > $(BUILDDIR)/.gobuildversion
	rm -f $(DESTDIR)/libwg.h


clean:
	rm -f $(DESTDIR)/libwg.so
