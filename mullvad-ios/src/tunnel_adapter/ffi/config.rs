//! The config-builder FFI: [`GotaTunConfigHandle`] and its `gotatun_config_*`
//! entry points, built up incrementally from Swift.

use super::super::{ObfuscationConfig, PeerConfig};
use super::{parse_cstr, read_32_bytes};
use std::net::{Ipv4Addr, SocketAddr};
use std::os::raw::c_char;

/// Opaque config handle built incrementally from Swift.
pub struct GotaTunConfigHandle {
    pub(super) private_key: [u8; 32],
    pub(super) ipv4_addr: Ipv4Addr,
    pub(super) mtu: u16,
    pub(super) exit_peer: PeerConfig,
    pub(super) entry_peer: Option<PeerConfig>,
    pub(super) ipv4_gateway: Ipv4Addr,
    pub(super) establish_timeout_secs: u32,
    pub(super) enable_pq: bool,
    pub(super) enable_daita: bool,
    pub(super) obfuscation: ObfuscationConfig,
}

/// Create a new config with the required private key, tunnel IPv4 address, and
/// exit peer.
///
/// Every tunnel must have an exit peer, so it is required at construction rather
/// than set separately — a config cannot exist without one.
///
/// Returns a handle that must be freed with `gotatun_config_free` or consumed
/// by `gotatun_start_tunnel`. Returns null if any argument is invalid.
///
/// # Safety
/// - `ipv4_addr` and `exit_endpoint` must be valid null-terminated C strings
///   (e.g. `exit_endpoint` like "1.2.3.4:51820").
/// - `private_key` and `exit_public_key` must each point to at least 32 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gotatun_config_new(
    private_key: *const u8,
    ipv4_addr: *const c_char,
    exit_public_key: *const u8,
    exit_endpoint: *const c_char,
) -> *mut GotaTunConfigHandle {
    let key = match read_32_bytes(private_key) {
        Some(k) => k,
        None => return std::ptr::null_mut(),
    };
    let addr: Ipv4Addr = match parse_cstr(ipv4_addr).and_then(|s| s.parse().ok()) {
        Some(a) => a,
        None => return std::ptr::null_mut(),
    };
    let exit_public_key = match read_32_bytes(exit_public_key) {
        Some(k) => k,
        None => return std::ptr::null_mut(),
    };
    let exit_endpoint: SocketAddr = match parse_cstr(exit_endpoint).and_then(|s| s.parse().ok()) {
        Some(ep) => ep,
        None => return std::ptr::null_mut(),
    };

    Box::into_raw(Box::new(GotaTunConfigHandle {
        private_key: key,
        ipv4_addr: addr,
        mtu: 1280,
        exit_peer: PeerConfig {
            public_key: exit_public_key,
            endpoint: exit_endpoint,
            allowed_ips: vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()],
        },
        entry_peer: None,
        ipv4_gateway: Ipv4Addr::new(10, 64, 0, 1),
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
    // SAFETY: per this function's `# Safety` contract, `config` is either null (then
    // `as_mut` yields `None`) or a valid, uniquely-owned pointer from
    // `gotatun_config_new`, so taking a mutable reference for this call is sound.
    if let Some(config) = unsafe { config.as_mut() } {
        config.mtu = mtu;
    }
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
    // SAFETY: per this function's `# Safety` contract, `config` is either null (then
    // `as_mut` yields `None`) or a valid, uniquely-owned pointer from
    // `gotatun_config_new`, so taking a mutable reference for this call is sound.
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
    // SAFETY: per this function's `# Safety` contract, `config` is either null (then
    // `as_mut` yields `None`) or a valid, uniquely-owned pointer from
    // `gotatun_config_new`, so taking a mutable reference for this call is sound.
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
    // SAFETY: per this function's `# Safety` contract, `config` is either null (then
    // `as_mut` yields `None`) or a valid, uniquely-owned pointer from
    // `gotatun_config_new`, so taking a mutable reference for this call is sound.
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
