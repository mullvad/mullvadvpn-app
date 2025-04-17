use super::{run_forwarding_proxy, ShadowsocksHandle};
use crate::api_client::helpers::parse_ip_addr;
use crate::ProxyHandle;
use std::net::SocketAddr;
#[cfg(any(target_os = "macos", target_os = "ios"))]
use std::sync::Once;

#[cfg(any(target_os = "macos", target_os = "ios"))]
static INIT_LOGGING: Once = Once::new();

/// # Safety
/// `addr`, `password`, `cipher` must be valid for the lifetime of this function call and they must
/// be backed by the amount of bytes as stored in the respective `*_len` parameters.
///
/// `proxy_config` must be pointing to a valid memory region for the size of a `ProxyHandle`
/// instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_shadowsocks_proxy(
    forward_address: *const u8,
    forward_address_len: usize,
    forward_port: u16,
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
            .level_filter(log::LevelFilter::Info)
            .init();
    });

    let forward_ip =
        if let Some(forward_address) = { parse_ip_addr(forward_address, forward_address_len) } {
            forward_address
        } else {
            return -1;
        };

    let forward_socket_addr = SocketAddr::new(forward_ip, forward_port);

    let bridge_ip = if let Some(addr) = { parse_ip_addr(addr, addr_len) } {
        addr
    } else {
        return -1;
    };

    let bridge_socket_addr = SocketAddr::new(bridge_ip, port);

    // SAFETY: See notes for `parse_str`
    let password = if let Some(password) = unsafe { parse_str(password, password_len) } {
        password
    } else {
        return -1;
    };

    // SAFETY: See notes for `parse_str`
    let cipher = if let Some(cipher) = unsafe { parse_str(cipher, cipher_len) } {
        cipher
    } else {
        return -1;
    };

    let (port, handle) =
        match run_forwarding_proxy(forward_socket_addr, bridge_socket_addr, &password, &cipher) {
            Ok((port, handle)) => (port, handle),
            Err(err) => {
                log::error!("Failed to run HTTP proxy {}", err);
                return err.raw_os_error().unwrap_or(-1);
            }
        };
    let handle = Box::new(handle);

    // SAFETY: `proxy_config` is guaranteed to be writeable for the duration of this call.
    // It does not overlap with `handle`
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
/// # Safety
/// `proxy_config` must be pointing to a valid instance of a `ProxyInstance`, as instantiated by
/// `start_shadowsocks_proxy`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stop_shadowsocks_proxy(proxy_config: *mut ProxyHandle) -> i32 {
    // SAFETY: `proxy_config` is guaranteed to be a valid pointer
    let context_ptr = unsafe { (*proxy_config).context };
    if context_ptr.is_null() {
        return -1;
    }

    // SAFETY: `context_ptr` is guaranteed to be a valid, non-null pointer
    let proxy_handle: Box<ShadowsocksHandle> = unsafe { Box::from_raw(context_ptr as *mut _) };
    proxy_handle.stop();
    // SAFETY: `proxy_config` is guaranteed to be a valid pointer
    unsafe { (*proxy_config).context = std::ptr::null_mut() };
    0
}

/// Allocates a new string with the contents of `data` if it contains only valid UTF-8 bytes.
///
/// SAFETY: `data` must be a valid pointer to `len` amount of bytes
unsafe fn parse_str(data: *const u8, len: usize) -> Option<String> {
    // SAFETY: data pointer must be valid for the size of len
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    String::from_utf8(bytes.to_vec()).ok()
}
