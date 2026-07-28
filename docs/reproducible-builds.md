# Reproducible builds

A build is reproducible when anyone can take the same source and produce byte-for-byte the same
binaries, no matter which machine they build on. That is what lets someone check that a released
binary really was built from the published source. See https://reproducible-builds.org/.

**Status:** Supported on Android, baby steps on Desktop, not on iOS

## Android

The Android app has been reproducible since `2025.2-beta1`, and its build instructions cover both
building reproducibly and verifying an official release:
[android/docs/BuildInstructions.md](../android/docs/BuildInstructions.md#reproducible-builds). The
rest of this document is about the desktop app.

## Desktop

The desktop releases of this app are not yet reproducible. But there has been work to eliminate
non-deterministic and machine dependent sections in the produced binaries.

The desktop binaries we release are **not** reproducible, and this document does not claim
otherwise. What works today is one step towards it: on a nightly toolchain, via
`building/reproducible-build.sh`, the Rust binaries no longer carry the file paths of the machine
that built them. We are not building releases that way, because we are not willing to compile the
software we ship with a nightly compiler.

Concretely, what is and is not in place:

- Machine specific paths in the Rust binaries: removed on nightly, see below.
- Other sources of nondeterminism in those binaries: only partly dealt with. The Windows PE
  timestamp is handled by the `/Brepro` link argument in `.cargo/config.toml`; the macOS Mach-O
  `LC_UUID`, which the linker derives from something other than the input, is not.
- The `.deb` and `.rpm` packages, the Windows installer and the macOS `.pkg`: not investigated in
  this repository yet.
- Publishing hashes, documenting a verification procedure, and checking reproducibility in CI: not
  done.

## How the Rust part works

Everything absolute that ends up in a binary comes from paths the compiler records: the source
directory, the cargo registry, the rustup sysroot, and the cargo target directory. The last one is
the one that is easy to miss. The gRPC bindings are generated into `OUT_DIR` at build time, and the
absolute path of that directory is baked into every panic location of the generated code, so two
builds from different directories produce different binaries even when the sources are identical.

Cargo's [`trim-paths`] replaces all four with fixed values. It is enabled in `.cargo/config.toml`:

```toml
[unstable]
trim-paths = true
```

Cargo then passes rustc a remapping for each of them, and the release profile, which is the only one
`trim-paths` touches by default, comes out free of machine specific paths:

```
$CARGO_HOME/registry/src               ->  (dependencies become foo-1.2.3/src/...)
$RUSTUP_HOME/.../lib/rustlib/src/rust  ->  /rustc/<commit hash>
<this repository>                      ->  .
<cargo target directory>               ->  /cargo/build-dir
```

The dev profile is left alone, so debuggers and backtraces still resolve sources in the builds where
that matters.

## Why it needs nightly

`trim-paths` is unstable, so only a nightly cargo acts on the key above. Stable cargo ignores the
whole `[unstable]` table, which is why the key can sit in a committed config file without affecting
the builds we ship, and why `rust-toolchain.toml` still pins a stable release.

`building/reproducible-build.sh` exists to bridge that. It sets [`RUSTUP_TOOLCHAIN`] to an exact
pinned nightly and runs `build.sh`. That variable [takes precedence over `rust-toolchain.toml`],
and is exported rather than passed as `cargo +toolchain` so that it also reaches the cargo
invocations the build does not make directly: the npm scripts that build the native node modules,
MSBuild, and cargo subcommands such as cargo-deb.

The nightly is pinned to an exact date rather than plain `nightly`, since a reproducible build has
to name its compiler. Recent rustup versions download it on first use. If yours does not, the
documented behaviour is that the toolchain has to exist already, so install it first:

```bash
rustup toolchain install nightly-2026-07-27
```

[`RUSTUP_TOOLCHAIN`]: https://rust-lang.github.io/rustup/environment-variables.html
[takes precedence over `rust-toolchain.toml`]: https://rust-lang.github.io/rustup/overrides.html

## Trying it

```bash
building/reproducible-build.sh --optimize
```

Build the same commit twice from two different directories and compare, for example with
`sha256sum` over `dist-assets/`. The Rust binaries should be identical on Linux and Windows. On
macOS they will still differ, because of the `LC_UUID` mentioned above.

## Feedback to the cargo project

`trim-paths` has been unstable since 2023. Part of the point of merging this is to use it on a real
code base and report back, so it can move towards stabilisation. If you hit something that does not
get trimmed, that is worth reporting on the tracking issue rather than working around here.

- Tracking issue: https://github.com/rust-lang/cargo/issues/12137
- RFC 3127: https://rust-lang.github.io/rfcs/3127-trim-paths.html

[`trim-paths`]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#profile-trim-paths-option
