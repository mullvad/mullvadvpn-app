#![cfg(any(target_os = "ios", target_os = "macos", target_os = "tvos"))]
// On macOS the iOS-only FFI exports are not compiled. We allow to build it on macOS to aid with
// running unit tests.
#![cfg_attr(target_os = "macos", allow(dead_code))]

mod gotatun;
mod tunnel_adapter;

// uniffi scaffolding for the gotatun FFI. Gated to `ios` to match the gotatun FFI
// module; the rest of the crate's FFI uses cbindgen and is unaffected.
#[cfg(any(target_os = "ios", target_os = "tvos"))]
uniffi::setup_scaffolding!("mullvad_gotatun");

#[cfg(any(target_os = "ios", target_os = "tvos"))]
mod api_client;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
mod ephemeral_peer_proxy;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
mod log_redactor;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
mod logging;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
mod tunnel_obfuscator_proxy;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
mod wireguard_key;

// --- iOS FFI glue (Swift interop) ---

#[cfg(any(target_os = "ios", target_os = "tvos"))]
use libc::c_char;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
use std::ffi::CStr;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
use std::sync::OnceLock;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
use tokio::runtime::{Builder, Handle, Runtime};

#[cfg(any(target_os = "ios", target_os = "tvos"))]
#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut std::ffi::c_void,
    pub port: u16,
}

#[cfg(any(target_os = "ios", target_os = "tvos"))]
#[unsafe(no_mangle)]
pub static CONFIG_SERVICE_PORT: u16 = talpid_tunnel_config_client::CONFIG_SERVICE_PORT;

#[cfg(any(target_os = "ios", target_os = "tvos"))]
static RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();

#[cfg(any(target_os = "ios", target_os = "tvos"))]
fn mullvad_ios_runtime() -> Result<Handle, String> {
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

#[cfg(any(target_os = "ios", target_os = "tvos"))]
/// Try to convert a C string to an owned [String]. if `ptr` is null, an empty [String] is
/// returned.
///
/// # Safety
/// - `ptr` must uphold all safety invariants as required by [CStr::from_ptr].
unsafe fn get_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // Safety: See function doc comment.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().map(ToOwned::to_owned).unwrap_or_default()
}
