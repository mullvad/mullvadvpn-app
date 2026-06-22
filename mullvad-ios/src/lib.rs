#![cfg(any(target_os = "ios", target_os = "macos"))]
// On macOS the iOS-only FFI exports are not compiled. We allow to build it on macOS to aid with
// running unit tests.
#![cfg_attr(target_os = "macos", allow(dead_code))]

mod gotatun;
mod tunnel_adapter;

// uniffi scaffolding for the gotatun FFI. Gated to `ios` to match the gotatun FFI
// module; the rest of the crate's FFI uses cbindgen and is unaffected.
#[cfg(target_os = "ios")]
uniffi::setup_scaffolding!("mullvad_gotatun");

#[cfg(target_os = "ios")]
mod api_client;
#[cfg(target_os = "ios")]
mod ephemeral_peer_proxy;
#[cfg(target_os = "ios")]
mod log_redactor;
#[cfg(target_os = "ios")]
mod logging;
#[cfg(target_os = "ios")]
mod tunnel_obfuscator_proxy;
#[cfg(target_os = "ios")]
mod wireguard_key;

// --- iOS FFI glue (Swift interop) ---

#[cfg(target_os = "ios")]
use libc::c_char;
#[cfg(target_os = "ios")]
use std::ffi::CStr;
#[cfg(target_os = "ios")]
use std::sync::OnceLock;
#[cfg(target_os = "ios")]
use tokio::runtime::{Builder, Handle, Runtime};

#[cfg(target_os = "ios")]
#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut std::ffi::c_void,
    pub port: u16,
}

#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub static CONFIG_SERVICE_PORT: u16 = talpid_tunnel_config_client::CONFIG_SERVICE_PORT;

#[cfg(target_os = "ios")]
static RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();

#[cfg(target_os = "ios")]
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

#[cfg(target_os = "ios")]
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
