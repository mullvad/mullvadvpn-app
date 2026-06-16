//! Tunnel lifecycle FFI: starting, stopping, and signalling a running tunnel.

use super::super::{IosTunnelAdapter, TunnelCallbackHandler, TunnelConfig};
use super::config::GotaTunConfigHandle;
use std::os::raw::c_char;
use std::sync::Arc;

/// Opaque handle to a running tunnel adapter.
pub struct GotaTunHandle {
    adapter: IosTunnelAdapter,
}

/// Callback function pointers from Rust to Swift.
#[repr(C)]
pub struct GotaTunCallbacks {
    /// Context pointer passed back to all callbacks.
    pub context: *mut std::ffi::c_void,
    /// Called when the tunnel is connected and traffic flows.
    pub on_connected: unsafe extern "C" fn(ctx: *mut std::ffi::c_void),
    /// Called when the pinger times out.
    pub on_timeout: unsafe extern "C" fn(ctx: *mut std::ffi::c_void),
    /// Called on fatal error. `message` is a null-terminated C string.
    pub on_error: unsafe extern "C" fn(ctx: *mut std::ffi::c_void, message: *const c_char),
}

// SAFETY: `GotaTunCallbacks` holds an opaque Swift context pointer and Swift function
// pointers. The Swift side guarantees the context is safe to send to another thread
// and keeps it alive until `gotatun_stop_tunnel`.
unsafe impl Send for GotaTunCallbacks {}
unsafe impl Sync for GotaTunCallbacks {}

impl TunnelCallbackHandler for GotaTunCallbacks {
    fn on_connected(&self) {
        unsafe { (self.on_connected)(self.context) };
    }

    fn on_timeout(&self) {
        unsafe { (self.on_timeout)(self.context) };
    }

    fn on_error(&self, message: String) {
        let c_msg = std::ffi::CString::new(message).unwrap_or_default();
        unsafe { (self.on_error)(self.context, c_msg.as_ptr()) };
    }
}

/// Start a GotaTun tunnel.
///
/// Consumes the config handle. Returns a tunnel handle, or null on error.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
/// - `callbacks.context` must remain valid until `gotatun_stop_tunnel`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_start_tunnel(
    tun_fd: i32,
    config: *mut GotaTunConfigHandle,
    callbacks: GotaTunCallbacks,
) -> *mut GotaTunHandle {
    if config.is_null() {
        return std::ptr::null_mut();
    }
    let config = unsafe { *Box::from_raw(config) };

    let runtime = match crate::mullvad_ios_runtime() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("Failed to get tokio runtime: {e}");
            return std::ptr::null_mut();
        }
    };

    let tunnel_config = TunnelConfig {
        tun_fd,
        private_key: config.private_key,
        ipv4_addr: config.ipv4_addr,
        ipv6_addr: None,
        mtu: config.mtu,
        exit_peer: config.exit_peer,
        entry_peer: config.entry_peer,
        ipv4_gateway: config.ipv4_gateway,
        establish_timeout_secs: config.establish_timeout_secs,
        enable_pq: config.enable_pq,
        enable_daita: config.enable_daita,
        obfuscation: config.obfuscation,
    };

    let adapter = IosTunnelAdapter::start(runtime, tunnel_config, Arc::new(callbacks));
    Box::into_raw(Box::new(GotaTunHandle { adapter }))
}

/// Stop and destroy a tunnel adapter.
///
/// # Safety
/// - `handle` must be a valid pointer from `gotatun_start_tunnel`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_stop_tunnel(handle: *mut GotaTunHandle) {
    if !handle.is_null() {
        let handle = unsafe { Box::from_raw(handle) };
        handle.adapter.stop();
    }
}

/// Recycle UDP sockets after a network path change.
///
/// # Safety
/// - `handle` must be a valid pointer from `gotatun_start_tunnel`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_recycle_sockets(handle: *mut GotaTunHandle) {
    if let Some(handle) = unsafe { handle.as_ref() } {
        handle.adapter.recycle_udp_sockets();
    }
}

/// Suspend the tunnel.
///
/// # Safety
/// - `handle` must be a valid pointer from `gotatun_start_tunnel`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_suspend_tunnel(handle: *mut GotaTunHandle) {
    if let Some(handle) = unsafe { handle.as_ref() } {
        handle.adapter.suspend();
    }
}

/// Wake the tunnel.
///
/// # Safety
/// - `handle` must be a valid pointer from `gotatun_start_tunnel`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_wake_tunnel(handle: *mut GotaTunHandle) {
    if let Some(handle) = unsafe { handle.as_ref() } {
        handle.adapter.wake();
    }
}
