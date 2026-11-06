#![cfg(any(target_os = "ios", target_os = "macos", target_os = "tvos"))]
// On macOS the iOS-only FFI exports are not compiled. We allow to build it on macOS to aid with
// running unit tests.
#![cfg_attr(target_os = "macos", allow(dead_code))]

#[cfg(any(target_os = "ios", target_os = "tvos"))]
mod ios;
mod tokio_bridge;
mod tunnel_adapter;

#[cfg(any(target_os = "ios", target_os = "tvos"))]
pub use ios::*;

use std::sync::OnceLock;
use tokio::runtime::{Builder, Handle, Runtime};

static RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();

pub fn mullvad_ios_runtime() -> Result<Handle, String> {
    match RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|error| ToString::to_string(&error))
    }) {
        Ok(runtime) => Ok(runtime.handle().clone()),
        Err(error) => Err(error.clone()),
    }
}
