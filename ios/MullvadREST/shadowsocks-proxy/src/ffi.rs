use super::{run_forwarding_proxy, ShadowsocksHandle};

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
#[cfg(any(target_os = "macos", target_os = "ios"))]
use std::sync::Once;

#[cfg(any(target_os = "macos", target_os = "ios"))]
const INIT_LOGGING: Once = Once::new();

#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut libc::c_void,
    pub port: u16,
}

#[no_mangle]
pub extern "C" fn return_42() -> i32 {
    42
}

#[no_mangle]
pub extern "C" fn start_shadowsocks_proxy(
    addr: *const u8,
    addr_len: usize,
    port: u16,
    password: *const u8,
    password_len: usize,
    cipher: *const u8,
    cipher_len: usize,
    proxy_config: *mut ProxyHandle,
) -> i32 {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    INIT_LOGGING.call_once(|| {
        let _ = oslog::OsLogger::new("net.mullvad.MullvadVPN.ShadowSocks")
            .level_filter(log::LevelFilter::Debug)
            .init();
    });

    let bridge_ip = if let Some(addr) = unsafe { parse_ip_addr(addr, addr_len) } {
        addr
    } else {
        return -1;
    };

    let bridge_addr = SocketAddr::new(bridge_ip, port);

    let password = if let Some(password) = unsafe { parse_str(password, password_len) } {
        password
    } else {
        return -1;
    };

    let cipher = if let Some(cipher) = unsafe { parse_str(cipher, cipher_len) } {
        cipher
    } else {
        return -1;
    };

    let (port, handle) = match run_forwarding_proxy(bridge_addr, &password, &cipher) {
        Ok((port, handle)) => (port, handle),
        Err(err) => {
            log::error!("Failed to run HTTP proxy {}", err);
            return err.raw_os_error().unwrap_or(-1);
        }
    };
    let handle = Box::new(handle);

    unsafe {
        std::ptr::write(
            proxy_config,
            ProxyHandle {
                port,
                context: Box::into_raw(handle) as *mut _,
            },
        )
    }

    0
}

#[no_mangle]
pub extern "C" fn stop_shadowsocks_proxy(proxy_config: *mut ProxyHandle) -> i32 {
    let context_ptr = unsafe { (*proxy_config).context };
    if context_ptr.is_null() {
        return -1;
    }

    let proxy_handle: Box<ShadowsocksHandle> = unsafe { Box::from_raw(context_ptr as *mut _) };
    proxy_handle.stop();
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
            addr_arr.as_mut_slice().copy_from_slice(&bytes);

            Some(Ipv6Addr::from(addr_arr).into())
        }
        anything_else => {
            log::error!("Bad IP address length {anything_else}");
            None
        }
    }
}

/// Allocates a new string with the contents of `data` if it contains only valid UTF-8 bytes.
///
/// SAFETY: `data` must be a valid pointer to `len` amount of bytes
unsafe fn parse_str(data: *const u8, len: usize) -> Option<String> {
    // SAFETY: data pointer must be valid for the size of len
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    String::from_utf8(bytes.to_vec()).ok()
}
