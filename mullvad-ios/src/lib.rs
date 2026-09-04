#![cfg(any(target_os = "ios", target_os = "macos", target_os = "tvos"))]
// On macOS the iOS-only FFI exports are not compiled. We allow to build it on macOS to aid with
// running unit tests.
#![cfg_attr(target_os = "macos", allow(dead_code))]

mod gotatun;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
mod ios;
mod tunnel_adapter;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
pub use ios::*;
