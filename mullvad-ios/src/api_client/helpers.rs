use std::{
    ffi::{CString, c_char, c_void},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

use shadowsocks::crypto::available_ciphers;
use talpid_types::net::proxy::{Shadowsocks, Socks5Remote, SocksAuth};

use super::get_string;

/// Constructs a new IP address from a pointer containing bytes representing an IP address.
///
/// SAFETY: `addr` pointer must be non-null, aligned, and point to at least addr_len bytes
pub(crate) unsafe fn parse_ip_addr(addr: *const u8, addr_len: usize) -> Option<IpAddr> {
    match addr_len {
        4 => {
            // SAFETY: `addr` pointer must be non-null, aligned, and point to at least addr_len bytes
            let bytes = unsafe { std::slice::from_raw_parts(addr, addr_len) };
            Some(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).into())
        }
        16 => {
            // SAFETY: `addr` pointer must be non-null, aligned, and point to at least addr_len bytes
            let bytes = unsafe { std::slice::from_raw_parts(addr, addr_len) };
            let mut addr_arr = [0u8; 16];
            addr_arr.as_mut_slice().copy_from_slice(bytes);

            Some(Ipv6Addr::from(addr_arr).into())
        }
        anything_else => {
            log::error!("Bad IP address length {anything_else}");
            None
        }
    }
}

/// Converts parameters into a boxed `Shadowsocks` configuration that is safe
/// to send across the FFI boundary
///
/// # SAFETY
/// `address` must be a pointer to at least `address_len` bytes.
/// `c_password` and `c_cipher` must be pointers to null terminated strings
#[unsafe(no_mangle)]
pub unsafe extern "C" fn new_shadowsocks_access_method_setting(
    address: *const u8,
    address_len: usize,
    port: u16,
    c_password: *const c_char,
    c_cipher: *const c_char,
) -> *const c_void {
    let endpoint: SocketAddr =
        if let Some(ip_address) = unsafe { parse_ip_addr(address, address_len) } {
            SocketAddr::new(ip_address, port)
        } else {
            return std::ptr::null();
        };

    let password = unsafe { get_string(c_password) };
    let cipher = unsafe { get_string(c_cipher) };

    let shadowsocks_configuration = Shadowsocks {
        endpoint,
        password,
        cipher,
    };

    Box::into_raw(Box::new(shadowsocks_configuration)) as *mut c_void
}

/// Converts parameters into a boxed `Socks5Remote` configuration that is safe
///
/// to send across the FFI boundary
///
/// # SAFETY
/// `address` must be a pointer to at least `address_len` bytes.
/// `c_username` and `c_password` must be pointers to null terminated strings, or null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn new_socks5_access_method_setting(
    address: *const u8,
    address_len: usize,
    port: u16,
    c_username: *const c_char,
    c_password: *const c_char,
) -> *const c_void {
    let endpoint: SocketAddr =
        if let Some(ip_address) = unsafe { parse_ip_addr(address, address_len) } {
            SocketAddr::new(ip_address, port)
        } else {
            return std::ptr::null();
        };

    let auth = {
        if c_username.is_null() || c_password.is_null() {
            None
        } else {
            let username = unsafe { get_string(c_username) };
            let password = unsafe { get_string(c_password) };
            SocksAuth::new(username, password).ok()
        }
    };

    let socks5_configuration = Socks5Remote { endpoint, auth };
    Box::into_raw(Box::new(socks5_configuration)) as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_shadowsocks_chipers() -> *mut libc::c_char {
    let ciphers_string = available_ciphers().join(",");
    let ciphers_c_string = CString::new(ciphers_string).unwrap_or_default();

    ciphers_c_string.into_raw()
}
