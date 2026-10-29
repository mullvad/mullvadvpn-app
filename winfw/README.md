# winfw

Windows firewall for the Mullvad app.

This crate is a work in progress. It currently contains:

* `guids`: the WFP object GUIDs shared between the C++ `winfw.dll` implementation
  and its Rust replacement,
* `cpp_ffi`: runtime bindings for the C++ implementation, used by the snapshot
  tests as the reference while the Rust implementation is written.

The goal is for this crate to fully replace the C++ code in `windows/winfw`.

## Snapshot tests

`tests/snapshot.rs` drives the firewall through every policy state and compares
the resulting WFP object state with the snapshots in `tests/snapshots/`. This
guards the Rust implementation against behavioral differences from the C++
implementation, feature by feature, until the latter can be deleted.

The tests talk to the real Base Filtering Engine, so they must run as
administrator, on Windows. They replace every WFP object the firewall owns,
including those of a running Mullvad daemon, so they are ignored by default:

```sh
WINFW_DLL_PATH=<path to winfw.dll> cargo test -p winfw --features cpp-ffi -- --ignored
```

The C++ DLL is built from `windows/winfw/winfw.sln` (see `build-windows-modules.sh`).

The snapshots are stored with [insta]. When one legitimately changes,
regenerate them with:

```sh
INSTA_UPDATE=always WINFW_DLL_PATH=<...> cargo test -p winfw --features cpp-ffi -- --ignored
```

or use the review workflow, which walks you through the differences one by
one:

```sh
WINFW_DLL_PATH=<...> cargo insta test -p winfw --features cpp-ffi -- --ignored
cargo insta review
```

Snapshots are normalized: object GUIDs are shown by name, interface LUIDs are
resolved back to their alias, the tunnel interface is shown as a placeholder,
application paths have their volume number replaced, the engine-set
`FWPM_FILTER_FLAG_INDEXED` bit is masked away, and filters and conditions are
sorted, since WFP does not assign meaning to their order.

[insta]: https://insta.rs
