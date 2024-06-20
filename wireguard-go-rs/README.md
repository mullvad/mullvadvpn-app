# `wireguard-go-rs`
This crate is a Rust-friendly wrapper around `wireguard-go`.
It wraps `libwg`, which in turn wraps [Mullvad VPN's fork of wireguard-go](https://github.com/mullvad/wireguard-go) that extends `wireguard-go` with [DAITA](https://mullvad.net/blog/introducing-defense-against-ai-guided-traffic-analysis-daita).

## Known limitation
To extend `wireguard-go` with DAITA capabilities, `wireguard-go` links against [maybenot](https://github.com/maybenot-io/maybenot/). This is done statically, which at the time of writing will cause issues if `wireguard-go` in turn is statically linked from another Rust crate: https://github.com/rust-lang/rust/issues/104707.
As such `libwg` is built as a shared object which you can link to dynamically, which circumvents this issue.
To get rid of this limitation, you can compile `wireguard-go` without DAITA support. See [build-wireguard-go.sh](./build-wireguard-go.sh) for details on how to do that.

## Upgrading `wireguard-go`
Upgrading `wireguard-go` involves updating the git submodule found in `libwg/wireguard-go`, which points to [Mullvad VPN's fork of wireguard-go](https://github.com/mullvad/wireguard-go). To update the fork, find the desired release of `wireguard-go` at <https://github.com/WireGuard/wireguard-go/tags> and rebase the fork on the corresponding commit. Change directory to `libwg` and run `go mod tidy` to update indirect dependencies.

To upgrade the version of `Go` run `go mod edit -go=XX`. You will also need to update the `ARG GOLANG_VERSION` version in `building/Dockerfile` and build and distribute new development container images, see the corresponding [instructions](../building/README.md).

Before upgrading the version of `Go` or `wireguard-go`, be aware that we depend on a patch for the internal clocks of the go runtime on android, see <https://git.zx2c4.com/wireguard-android/tree/tunnel/tools/libwg-go>. Upgrading the versions of `wireguard-go` or `Go` beyond what the patch is built for should be done with caution. Note, however, that the patch states that "In Linux 4.17, the kernel will actually make MONOTONIC act like BOOTTIME anyway, so this switch will additionally unify the timer behavior across kernels." According to <https://source.android.com/docs/core/architecture/kernel/android-common>, Android version 11 and newer seem to use sufficiently new versions of the linux kernel to not need this patch. When we no longer support older versions of android, we may be able to drop this compatibility requirement.