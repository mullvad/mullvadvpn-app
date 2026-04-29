//! C FFI for the iOS tunnel adapter.
//!
//! Config is built using the builder pattern:
//! 1. `gotatun_config_new(private_key, ipv4_addr)` → config handle
//! 2. `gotatun_config_set_*()` — set additional params
//! 3. `gotatun_start_tunnel(tun_fd, config, callbacks)` → tunnel handle
//! 4. `gotatun_stop_tunnel(handle)` — stop and free

use super::{IosTunnelAdapter, ObfuscationConfig, PeerConfig, TunnelCallbackHandler, TunnelConfig};
use std::{
    ffi::CStr,
    net::{Ipv4Addr, SocketAddr},
    os::raw::c_char,
    sync::Arc,
};

// =============================================================================
// Config builder
// =============================================================================

/// Opaque config handle built incrementally from Swift.
pub struct GotaTunConfigHandle {
    private_key: [u8; 32],
    ipv4_addr: Ipv4Addr,
    mtu: u16,
    exit_peer: Option<PeerConfig>,
    entry_peer: Option<PeerConfig>,
    ipv4_gateway: Ipv4Addr,
    retry_attempt: u32,
    establish_timeout_secs: u32,
    enable_pq: bool,
    enable_daita: bool,
    obfuscation: ObfuscationConfig,
}

/// Create a new config with the required private key and tunnel IPv4 address.
///
/// Returns a handle that must be freed with `gotatun_config_free` or consumed
/// by `gotatun_start_tunnel`.
///
/// # Safety
/// - `ipv4_addr` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_new(
    private_key: *const u8,
    ipv4_addr: *const c_char,
) -> *mut GotaTunConfigHandle {
    let key = match read_32_bytes(private_key) {
        Some(k) => k,
        None => return std::ptr::null_mut(),
    };
    let addr: Ipv4Addr = match parse_cstr(ipv4_addr).and_then(|s| s.parse().ok()) {
        Some(a) => a,
        None => return std::ptr::null_mut(),
    };

    Box::into_raw(Box::new(GotaTunConfigHandle {
        private_key: key,
        ipv4_addr: addr,
        mtu: 1280,
        exit_peer: None,
        entry_peer: None,
        ipv4_gateway: Ipv4Addr::new(10, 64, 0, 1),
        retry_attempt: 0,
        establish_timeout_secs: 4,
        enable_pq: false,
        enable_daita: false,
        obfuscation: ObfuscationConfig::Off,
    }))
}

/// Set the MTU on the config.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_mtu(config: *mut GotaTunConfigHandle, mtu: u16) {
    if let Some(config) = unsafe { config.as_mut() } {
        config.mtu = mtu;
    }
}

/// Set the exit peer on the config.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
/// - `public_key` must point to at least 32 bytes.
/// - `endpoint` must be a valid null-terminated C string (e.g. "1.2.3.4:51820").
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_exit_peer(
    config: *mut GotaTunConfigHandle,
    public_key: *const u8,
    endpoint: *const c_char,
) {
    let Some(config) = (unsafe { config.as_mut() }) else {
        return;
    };
    let Some(key) = read_32_bytes(public_key) else {
        return;
    };
    let Some(ep) = parse_cstr(endpoint).and_then(|s| s.parse::<SocketAddr>().ok()) else {
        return;
    };

    config.exit_peer = Some(PeerConfig {
        public_key: key,
        endpoint: ep,
        allowed_ips: vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()],
    });
}

/// Set the entry peer for multihop.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
/// - `public_key` must point to at least 32 bytes.
/// - `endpoint` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_entry_peer(
    config: *mut GotaTunConfigHandle,
    public_key: *const u8,
    endpoint: *const c_char,
) {
    let Some(config) = (unsafe { config.as_mut() }) else {
        return;
    };
    let Some(key) = read_32_bytes(public_key) else {
        return;
    };
    let Some(ep) = parse_cstr(endpoint).and_then(|s| s.parse::<SocketAddr>().ok()) else {
        return;
    };

    config.entry_peer = Some(PeerConfig {
        public_key: key,
        endpoint: ep,
        allowed_ips: vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()],
    });
}

/// Set the gateway IPv4 address used for connectivity pings.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
/// - `gateway` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_gateway(
    config: *mut GotaTunConfigHandle,
    gateway: *const c_char,
) {
    let Some(config) = (unsafe { config.as_mut() }) else {
        return;
    };
    if let Some(gw) = parse_cstr(gateway).and_then(|s| s.parse().ok()) {
        config.ipv4_gateway = gw;
    }
}

/// Enable post-quantum key exchange.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_enable_pq(config: *mut GotaTunConfigHandle) {
    if let Some(config) = unsafe { config.as_mut() } {
        config.enable_pq = true;
    }
}

/// Enable DAITA.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_enable_daita(config: *mut GotaTunConfigHandle) {
    if let Some(config) = unsafe { config.as_mut() } {
        config.enable_daita = true;
    }
}

/// Set obfuscation to UDP-over-TCP.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_obfuscation_udp_over_tcp(
    config: *mut GotaTunConfigHandle,
) {
    if let Some(config) = unsafe { config.as_mut() } {
        config.obfuscation = ObfuscationConfig::UdpOverTcp;
    }
}

/// Set obfuscation to Shadowsocks.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_obfuscation_shadowsocks(
    config: *mut GotaTunConfigHandle,
) {
    if let Some(config) = unsafe { config.as_mut() } {
        config.obfuscation = ObfuscationConfig::Shadowsocks;
    }
}

/// Set obfuscation to QUIC.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
/// - `hostname` and `token` must be valid null-terminated C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_obfuscation_quic(
    config: *mut GotaTunConfigHandle,
    hostname: *const c_char,
    token: *const c_char,
) {
    let Some(config) = (unsafe { config.as_mut() }) else {
        return;
    };
    let Some(hostname) = parse_cstr(hostname) else {
        return;
    };
    let Some(token) = parse_cstr(token) else {
        return;
    };
    config.obfuscation = ObfuscationConfig::Quic { hostname, token };
}

/// Set obfuscation to LWO (Lightweight Obfuscation).
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
/// - `client_public_key` and `server_public_key` must point to at least 32 bytes each.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_obfuscation_lwo(
    config: *mut GotaTunConfigHandle,
    client_public_key: *const u8,
    server_public_key: *const u8,
) {
    let Some(config) = (unsafe { config.as_mut() }) else {
        return;
    };
    let Some(client_key) = read_32_bytes(client_public_key) else {
        return;
    };
    let Some(server_key) = read_32_bytes(server_public_key) else {
        return;
    };
    config.obfuscation = ObfuscationConfig::Lwo {
        client_public_key: client_key,
        server_public_key: server_key,
    };
}

/// Set the retry attempt number (affects establish timeout).
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_retry_attempt(
    config: *mut GotaTunConfigHandle,
    retry_attempt: u32,
) {
    if let Some(config) = unsafe { config.as_mut() } {
        config.retry_attempt = retry_attempt;
    }
}

/// Set the establish timeout in seconds.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_set_establish_timeout(
    config: *mut GotaTunConfigHandle,
    timeout_secs: u32,
) {
    if let Some(config) = unsafe { config.as_mut() } {
        config.establish_timeout_secs = timeout_secs;
    }
}

/// Free a config without starting a tunnel.
///
/// # Safety
/// - `config` must be a valid pointer from `gotatun_config_new`.
/// - Must only be called once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_free(config: *mut GotaTunConfigHandle) {
    if !config.is_null() {
        drop(unsafe { Box::from_raw(config) });
    }
}

// =============================================================================
// Tunnel lifecycle
// =============================================================================

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

    let exit_peer = match config.exit_peer {
        Some(p) => p,
        None => {
            log::error!("No exit peer configured");
            return std::ptr::null_mut();
        }
    };

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
        exit_peer,
        entry_peer: config.entry_peer,
        ipv4_gateway: config.ipv4_gateway,
        retry_attempt: config.retry_attempt,
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

// =============================================================================
// Helpers
// =============================================================================

fn parse_cstr(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(String::from)
}

fn read_32_bytes(ptr: *const u8) -> Option<[u8; 32]> {
    if ptr.is_null() {
        return None;
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, 32) };
    let mut arr = [0u8; 32];
    arr.copy_from_slice(slice);
    Some(arr)
}
