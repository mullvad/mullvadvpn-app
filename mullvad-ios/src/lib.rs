#![cfg(target_os = "ios")]
#![allow(clippy::undocumented_unsafe_blocks)]
use libc::c_char;
use std::ffi::CStr;
use std::sync::OnceLock;
use tokio::runtime::{Builder, Handle, Runtime};

mod api_client;
mod ephemeral_peer_proxy;
pub mod tunnel_obfuscator_proxy;

#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut std::ffi::c_void,
    pub port: u16,
}

#[unsafe(no_mangle)]
pub static CONFIG_SERVICE_PORT: u16 = talpid_tunnel_config_client::CONFIG_SERVICE_PORT;

static RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();

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
