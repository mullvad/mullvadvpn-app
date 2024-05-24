# `wireguard-go-rs`
This crate is a Rust-friendly wrapper around `wireguard-go`.
It wraps `libwg`, which in turn wraps [Mullvad VPN's fork of wireguard-go](https://github.com/mullvad/wireguard-go) that extends `wireguard-go` with [DAITA](https://mullvad.net/en/blog/introducing-defense-against-ai-guided-traffic-analysis-daita).

## Known limitation
To extend `wireguard-go` with DAITA capabilities, `wireguard-go` links against [maybenot](https://github.com/maybenot-io/maybenot/). This is done statically, which at the time of writing will cause issues if `wireguard-go` in turn is statically linked from another Rust crate: https://github.com/rust-lang/rust/issues/104707.
As such `libwg` is built as a shared object which you can link to dynamically, which circumvents this issue.
To get rid of this limitation, you can compile `wireguard-go` without DAITA support. See [build-wireguard-go.sh](./build-wireguard-go.sh) for details on how to do that.

## Upgrading `wireguard-go`
Upgrading `wireguard-go` involves updating the git submodule found in `libwg/wireguard-go`. This module uses [Mullvad VPN's fork of wireguard-go](https://github.com/mullvad/wireguard-go).
