use libc::c_char;

use super::{TunnelObfuscatorHandle, TunnelObfuscatorRuntime};
use crate::ProxyHandle;
use std::{net::SocketAddr, sync::Once};

use crate::{api_client::helpers::parse_ip_addr, get_string};

static INIT_LOGGING: Once = Once::new();

macro_rules! throw_int_error {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(value) => return value,
        }
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_udp2tcp_obfuscator_proxy(
    peer_address: *const u8,
    peer_address_len: usize,
    peer_port: u16,
    proxy_handle: *mut ProxyHandle,
) -> i32 {
    init_logging();

    let peer_sock_addr = throw_int_error!(get_socket_address(peer_address, peer_address_len, peer_port));
    let result = TunnelObfuscatorRuntime::new_udp2tcp(peer_sock_addr).run();

    start(proxy_handle, result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_shadowsocks_obfuscator_proxy(
    peer_address: *const u8,
    peer_address_len: usize,
    peer_port: u16,
    proxy_handle: *mut ProxyHandle,
) -> i32 {
    init_logging();

    let peer_sock_addr = throw_int_error!(get_socket_address(peer_address, peer_address_len, peer_port));
    let result = TunnelObfuscatorRuntime::new_shadowsocks(peer_sock_addr).run();

    start(proxy_handle, result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_quic_obfuscator_proxy(
    peer_address: *const u8,
    peer_address_len: usize,
    peer_port: u16,
    hostname: *const c_char,
    token: *const c_char,
    proxy_handle: *mut ProxyHandle,
) -> i32 {
    init_logging();

    let peer_sock_addr = throw_int_error!(get_socket_address(peer_address, peer_address_len, peer_port));
    let hostname = get_string(hostname);
    let token = get_string(token);
    let result = TunnelObfuscatorRuntime::new_quic(peer_sock_addr, hostname, token).run();

    start(proxy_handle, result)
}

fn init_logging() {
    INIT_LOGGING.call_once(|| {
        let _ = oslog::OsLogger::new("net.mullvad.MullvadVPN.TunnelObfuscatorProxy")
            .level_filter(log::LevelFilter::Info)
            .init();
    });
}

/// Constructs a new IP address from a pointer containing bytes representing an IP address.
///
/// SAFETY: `addr` pointer must be non-null, aligned, and point to at least addr_len bytes
unsafe fn get_socket_address(
    peer_address: *const u8,
    peer_address_len: usize,
    peer_port: u16,
) -> Result<SocketAddr, i32> {
    let peer_sock_addr =
        // SAFETY: See notes for `parse_ip_addr`.
        if let Some(ip_address) = unsafe { parse_ip_addr(peer_address, peer_address_len) } {
            SocketAddr::new(ip_address, peer_port)
        } else {
            return Err(-1);
        };
    Ok(peer_sock_addr)
}

/// # Safety
///
/// Behavior is undefined if any of the following conditions are violated:
///
/// * `proxy_handle` must be [valid] for writes.
/// * `proxy_handle` must be properly aligned. Use [`write_unaligned`] if this is not the
///   case.
unsafe fn start(
    proxy_handle: *mut ProxyHandle,
    result: Result<(SocketAddr, TunnelObfuscatorHandle), std::io::Error>,
) -> i32 {
    match result {
        Ok((local_endpoint, obfuscator_handle)) => {
            let boxed_handle = Box::new(obfuscator_handle);
            let source_handle = ProxyHandle {
                context: Box::into_raw(boxed_handle) as *mut _,
                port: local_endpoint.port(),
            };
            
            // SAFETY: Caller guarantees that this pointer is valid.
            unsafe {
                std::ptr::write(
                    proxy_handle,
                    source_handle
                )
            };

            0
        }
        Err(err) => {
            log::error!("Failed to run tunnel obfuscator proxy {}", err);
            err.raw_os_error().unwrap_or(-1)
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stop_tunnel_obfuscator_proxy(proxy_handle: *mut ProxyHandle) -> i32 {
    // SAFETY: `proxy_config` is guaranteed to be a valid pointer
    let context_ptr = unsafe { (*proxy_handle).context };
    if context_ptr.is_null() {
        return -1;
    }

    // SAFETY: `context_ptr` is guaranteed to be a valid, non-null pointer
    let obfuscator_handle: Box<TunnelObfuscatorHandle> =
        unsafe { Box::from_raw(context_ptr as *mut _) };
    obfuscator_handle.stop();
    // SAFETY: `proxy_config` is guaranteed to be a valid pointer
    unsafe { (*proxy_handle).context = std::ptr::null_mut() };
    0
}
