#!/usr/bin/env bash

# Overrides the Rust toolchain in rust-toolchain.toml with a pinned nightly toolchain,
# and runs build.sh, passing on the arguments given here.
#
# Unstable `trim-paths` keep machine specific paths out of the Rust binaries.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

# An exact nightly rather than plain "nightly", so that everyone rebuilding a given commit
# uses the same compiler. Bumping it changes the bytes that come out.
# Takes precedence over rust-toolchain.toml, and is inherited by the cargo invocations we
# do not make ourselves: npm's native module builds, MSBuild and cargo-deb.
# https://rust-lang.github.io/rustup/overrides.html
export RUSTUP_TOOLCHAIN="nightly-2026-09-15"

# Makes rustc record fixed file paths instead of machine specific ones. The `[unstable]` table in
# .cargo/config.toml only enables the feature; since the release profile lost its default scope in
# https://github.com/rust-lang/cargo/pull/17424 the scope has to be set per profile as well.
# It cannot go in .cargo/config.toml, because stable cargo rejects the key instead of ignoring it,
# so it is set here, where nightly is already pinned. Profiles inheriting `release`, such as
# `release-debuginfo` and `installer-downloader`, inherit the scope.
export CARGO_PROFILE_RELEASE_TRIM_PATHS="object"

echo "Building with $RUSTUP_TOOLCHAIN for reproducibility. Not a release build."

exec "$REPO_DIR/build.sh" "$@"
