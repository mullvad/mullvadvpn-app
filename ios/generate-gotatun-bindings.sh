#!/usr/bin/env bash
#
# Regenerate the uniffi Swift bindings for the gotatun FFI.
#
# uniffi-bindgen runs in "library mode": it reads metadata embedded in the
# compiled staticlib, so the lib must be built first (chicken-and-egg — this is
# why generation is NOT done from build.rs). The generated artifacts are checked
# into the repo, mirroring the cbindgen-generated mullvad_rust_runtime.h.
#
# Outputs:
#   ios/MullvadRustRuntime/generated/mullvad_gotatun.swift   (Swift bindings)
#   ios/MullvadRustRuntime/include/mullvad_gotatunFFI.h       (C scaffolding header)
#
# The Swift bindings `import MullvadRustRuntimeProxy` (the existing private
# framework module), so the FFI header is folded into that module via
# ios/MullvadRustRuntime/module.private.modulemap. uniffi names the header after
# `ffi_module_name` (MullvadRustRuntimeProxy.h); we rename it to mullvad_gotatunFFI.h
# to sit unambiguously beside the cbindgen header. The import is module-based, so
# the header filename is free to change.

set -euo pipefail

TARGET="${1:-aarch64-apple-ios}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

LIB="target/$TARGET/debug/libmullvad_ios.a"
OUT_DIR="ios/MullvadRustRuntime/generated"

echo "Building libmullvad_ios for $TARGET..."
cargo build -p mullvad-ios --lib --target "$TARGET"

echo "Generating Swift bindings from $LIB..."
cargo run -p mullvad-ios --features uniffi-cli --bin uniffi-bindgen -- \
    generate \
    --library "$LIB" \
    --language swift \
    --config mullvad-ios/uniffi.toml \
    --out-dir "$OUT_DIR"

# uniffi names the FFI header after `ffi_module_name`; rename it and place it in the
# include dir alongside the cbindgen header so the framework module exposes it.
mv "$OUT_DIR/MullvadRustRuntimeProxy.h" ios/MullvadRustRuntime/include/mullvad_gotatunFFI.h

echo "Done. Generated:"
echo "  $OUT_DIR/mullvad_gotatun.swift"
echo "  ios/MullvadRustRuntime/include/mullvad_gotatunFFI.h"
