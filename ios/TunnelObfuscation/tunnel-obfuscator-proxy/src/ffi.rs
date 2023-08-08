#![cfg(target_os = "ios")]

use super::{TunnelObfuscatorHandle, TunnelObfuscatorRuntime};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Once,
};

static INIT_LOGGING: Once = Once::new();

#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut std::ffi::c_void,
    pub port: u16,
}

#[no_mangle]
pub unsafe extern "C" fn start_tunnel_obfuscator_proxy(
    peer_address: *const u8,
    peer_address_len: usize,
    peer_port: u16,
    proxy_handle: *mut ProxyHandle,
) -> i32 {
    INIT_LOGGING.call_once(|| {
        let _ = oslog::OsLogger::new("net.mullvad.MullvadVPN.TunnelObfuscatorProxy")
            .level_filter(log::LevelFilter::Info)
            .init();
    });

    let peer_sock_addr: SocketAddr =
        if let Some(ip_address) = parse_ip_addr(peer_address, peer_address_len) {
            SocketAddr::new(ip_address, peer_port)
        } else {
            return -1;
        };

    let result = TunnelObfuscatorRuntime::new(peer_sock_addr).and_then(|runtime| runtime.run());

    match result {
        Ok((local_endpoint, obfuscator_handle)) => {
            let boxed_handle = Box::new(obfuscator_handle);
            std::ptr::write(
                proxy_handle,
                ProxyHandle {
                    context: Box::into_raw(boxed_handle) as *mut _,
                    port: local_endpoint.port(),
                },
            );
            0
        }
        Err(err) => {
            log::error!("Failed to run tunnel obfuscator proxy {}", err);
            err.raw_os_error().unwrap_or(-1)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn stop_tunnel_obfuscator_proxy(proxy_handle: *mut ProxyHandle) -> i32 {
    let context_ptr = unsafe { (*proxy_handle).context };
    if context_ptr.is_null() {
        return -1;
    }

    let obfuscator_handle: Box<TunnelObfuscatorHandle> =
        unsafe { Box::from_raw(context_ptr as *mut _) };
    obfuscator_handle.stop();
    unsafe { (*proxy_handle).context = std::ptr::null_mut() };
    0
}

/// Constructs a new IP address from a pointer containing bytes representing an IP address.
///
/// SAFETY: `addr` must be a pointer to at least `addr_len` bytes.
unsafe fn parse_ip_addr(addr: *const u8, addr_len: usize) -> Option<IpAddr> {
    match addr_len {
        4 => {
            // SAFETY: addr pointer must point to at least addr_len bytes
            let bytes = unsafe { std::slice::from_raw_parts(addr, addr_len) };
            Some(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).into())
        }
        16 => {
            // SAFETY: addr pointer must point to at least addr_len bytes
            let bytes = unsafe { std::slice::from_raw_parts(addr, addr_len) };
            let mut addr_arr = [0u8; 16];
            addr_arr.as_mut_slice().copy_from_slice(bytes);

            Some(Ipv6Addr::from(addr_arr).into())
        }
        _ => None,
    }
}
