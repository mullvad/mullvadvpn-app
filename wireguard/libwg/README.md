# Introduction

`libwg` is a tiny wrapper around `wireguard-go`, with the main purpose of providing a simple FFI-friendly interface.

It currently offers support for the following platforms:

- Linux
- macOS
- Android

# Organization

`libwg.go` has shared code that is used on all platforms.

`libwg_default.go` has default implementations for Linux-based systems.

`libwg_android.go` has code specifically for Android.

# Usage

Call `wgTurnOn` to create and activate a tunnel. The prototype is different on different platforms, see the code for details.

Call `wgTurnOff` to destroy the tunnel.
