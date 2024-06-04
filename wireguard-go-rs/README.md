# `wireguard-go-rs`
This crate provides a Rust wrapper around `wireguard-go`.

## Known limitation
This crate wraps [Mullvad VPN's fork of wireguard-go](https://github.com/mullvad/wireguard-go), which extends `wireguard-go` with DAITA.
To do this, `wireguard-go` statically links against [maybenot](https://github.com/maybenot-io/maybenot/), which at the time of writing will cause issues if it in turn is statically linked from another Rust crate: https://github.com/rust-lang/rust/issues/104707.
As such, `libwg` is built as a shared object by default, which you have to link to dynamically.
To get rid of this limitation, you could compile `wireguard-go` without DAITA support.

## Upgrading `wireguard-go`
Upgrading `wireguard-go` involves updating the git submodule found in `libwg/wireguard-go`. This module uses [Mullvad VPN's fork of wireguard-go](https://github.com/mullvad/wireguard-go).
