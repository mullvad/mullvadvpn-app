# SPDX-License-Identifier: GPL-2.0
#
# Copyright (C) 2018-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.

# These are generally passed to us by xcode, but we set working defaults for standalone compilation too.
ARCHS ?= arm64 armv7
SDK_NAME ?= iphoneos
SDKROOT ?= $(shell xcrun --sdk $(SDK_NAME) --show-sdk-path)
CONFIGURATION_BUILD_DIR ?= $(CURDIR)/out
CONFIGURATION_TEMP_DIR ?= $(CURDIR)/.tmp

export CC ?= clang
LIPO ?= lipo
DESTDIR ?= $(CONFIGURATION_BUILD_DIR)
BUILDDIR ?= $(CONFIGURATION_TEMP_DIR)/wireguard-go-bridge

UPSTREAM_FILES := $(filter-out %/main.go %/queueconstants.go,$(wildcard ../wireguard-go/*/*.go) $(wildcard ../wireguard-go/*.go)) ../wireguard-go/go.mod ../wireguard-go/go.sum
DOWNSTREAM_FILES := $(wildcard src/*.go) $(wildcard src/*/*.go)
CFLAGS_PREFIX := $(if $(DEPLOYMENT_TARGET_CLANG_FLAG_NAME),-$(DEPLOYMENT_TARGET_CLANG_FLAG_NAME)=$($(DEPLOYMENT_TARGET_CLANG_ENV_NAME)),) -Wno-unused-command-line-argument -isysroot $(SDKROOT) -arch
GOARCH_arm64 := arm64
GOARCH_armv7 := arm
GOARCH_x86_64 := amd64
export GOOS := darwin
export CGO_ENABLED := 1

build: $(DESTDIR)/libwg-go.a
version-header: $(DESTDIR)/wireguard-go-version.h

GOBUILDVERSION_NEEDED := go version go1.12.5 darwin/amd64
GOBUILDVERSION_CURRENT := $(shell go version 2>/dev/null)
export REAL_GOROOT := $(shell go env GOROOT 2>/dev/null)
export GOROOT := $(BUILDDIR)/goroot
export GOPATH := $(BUILDDIR)/gopath
export PATH := $(GOPATH)/bin:$(PATH)
GOBUILDVERSION_FAKE := $(shell $(GOROOT)/bin/go version 2>/dev/null)
ifneq ($(GOBUILDVERSION_NEEDED),$(GOBUILDVERSION_CURRENT))
$(error This requires $(GOBUILDVERSION_NEEDED))
endif
ifneq ($(GOBUILDVERSION_NEEDED),$(GOBUILDVERSION_FAKE))
$(shell rm -f $(GOROOT)/.prepared)
endif

$(GOROOT)/.prepared:
	[ -n "$(REAL_GOROOT)" ]
	mkdir -p "$(GOROOT)"
	rsync -a --delete --exclude=pkg/obj/go-build "$(REAL_GOROOT)/" "$(GOROOT)/"
	cat goruntime-*.diff | patch -p1 -f -N -r- -d "$(GOROOT)"
	rm -rf "$(GOPATH)/pkg/mod"
	go get -d -tags ios; chmod -fR +w "$(GOPATH)/pkg/mod"
	for sys in "$(GOPATH)/pkg/mod/golang.org/x/sys@"*; do cat sys-unix-*.diff | patch -p1 -f -N -r- -d "$$sys"; done
	touch "$@"

$(shell test "$$(cat "$(BUILDDIR)/.gobuildversion" 2>/dev/null)" = "$(GOBUILDVERSION_CURRENT)" || rm -f "$(DESTDIR)/libwg-go.a")

define libwg-go-a
$(BUILDDIR)/libwg-go-$(1).a: $(GOROOT)/.prepared
	CGO_CFLAGS="$(CFLAGS_PREFIX) $(ARCH)" \
	CGO_LDFLAGS="$(CFLAGS_PREFIX) $(ARCH)" \
	GOARCH="$(GOARCH_$(1))" \
	go build -tags ios -ldflags=-w -v -o "$(BUILDDIR)/libwg-go-$(1).a" -buildmode c-archive && go version > "$(BUILDDIR)/.gobuildversion"; \
	chmod -fR +w "$(GOPATH)/pkg/mod"; \
	ret=$$$$?; \
	rm -f "$(BUILDDIR)/libwg-go-$(1).h"; \
	exit $$$$ret
endef
$(foreach ARCH,$(ARCHS),$(eval $(call libwg-go-a,$(ARCH))))

$(DESTDIR)/wireguard-go-version.h: go.mod $(GOROOT)/.prepared
	wggo="$(GOPATH)/pkg/mod/$$(sed -n 's/.*\(golang\.zx2c4\.com\/wireguard\) \(.*\)$$/\1@\2/p' go.mod)"; \
	sed -n 's/.*WireGuardGoVersion = "\(.*\)"/#define WIREGUARD_GO_VERSION "\1"/p' "$$wggo/device/version.go" > "$@"

$(DESTDIR)/libwg-go.a: $(foreach ARCH,$(ARCHS),$(BUILDDIR)/libwg-go-$(ARCH).a)
	@mkdir -vp "$(DESTDIR)"
	$(LIPO) -create -output "$@" $^

clean:
	rm -rf "$(BUILDDIR)" "$(DESTDIR)/libwg-go.a" "$(DESTDIR)/wireguard-go-version.h"

install: build

.PHONY: clean build version-header install
