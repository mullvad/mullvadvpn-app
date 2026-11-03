#![cfg(any(target_os = "ios", target_os = "tvos"))]

// uniffi scaffolding for the FFI. Gated to `ios` to match the FFI
// module; the rest of the crate's FFI uses cbindgen and is unaffected.
uniffi::setup_scaffolding!("mullvad_uniffi");

pub mod api_client;
pub mod ephemeral_peer_proxy;
pub mod log_redactor;
pub mod logging;
pub mod tunnel_obfuscator_proxy;
pub mod type_bridges;
pub mod wireguard_key;

// --- iOS FFI glue (Swift interop) ---

use libc::c_char;
use std::ffi::CStr;

#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut std::ffi::c_void,
    pub port: u16,
}

#[unsafe(no_mangle)]
pub static CONFIG_SERVICE_PORT: u16 = talpid_tunnel_config_client::CONFIG_SERVICE_PORT;

/// Try to convert a C string to an owned [String]. if `ptr` is null, an empty [String] is
/// returned.
///
/// # Safety
/// - `ptr` must uphold all safety invariants as required by [CStr::from_ptr].
pub unsafe fn get_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // Safety: See function doc comment.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().map(ToOwned::to_owned).unwrap_or_default()
}
