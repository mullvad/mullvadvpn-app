use std::{
    ffi::{CString, c_char, c_void},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

use shadowsocks::crypto::available_ciphers;
use talpid_types::net::proxy::{Shadowsocks, ShadowsocksCipher, Socks5Remote, SocksAuth};

use crate::api_client::access_method_settings::{ShadowsocksWrapper, Socks5RemoteWrapper};

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
#[uniffi::export]
pub fn new_shadowsocks_access_method_setting(
    address: Vec<u8>,
    port: u16,
    password: String,
    cipher: String,
) -> ShadowsocksWrapper {
    let endpoint: SocketAddr =
        // SAFETY: `addr` pointer must be non-null, aligned, and point to at least addr_len bytes
        if let Some(ip_address) = unsafe { parse_ip_addr(address.as_ptr(), address.len()) } {
            SocketAddr::new(ip_address, port)
        } else {
            todo!()
        };

    let cipher = ShadowsocksCipher::new(&cipher).unwrap();

    let shadowsocks_configuration = Shadowsocks::new(endpoint, cipher, password);

    ShadowsocksWrapper(shadowsocks_configuration)
}

/// Converts parameters into a boxed `Socks5Remote` configuration that is safe
///
/// to send across the FFI boundary
///
/// # SAFETY
/// `address` must be a pointer to at least `address_len` bytes.
/// `c_username` and `c_password` must be pointers to null terminated strings, or null
#[uniffi::export]
pub fn new_socks5_access_method_setting(
    address: Vec<u8>,
    port: u16,
    username: Option<String>,
    password: Option<String>,
) -> Socks5RemoteWrapper {
    let endpoint: SocketAddr =
        // SAFETY: caller guarantees that `address` is valid for at least `address_len` bytes
        if let Some(ip_address) = unsafe { parse_ip_addr(address.as_ptr(), address.len()) } {
            SocketAddr::new(ip_address, port)
        } else {
            panic!()
        };

    let auth = SocksAuth::new(username.unwrap_or_default(), password.unwrap_or_default()).ok();

    let socks5_configuration = Socks5Remote { endpoint, auth };
    Socks5RemoteWrapper(socks5_configuration)
}

#[unsafe(no_mangle)]
pub extern "C" fn get_shadowsocks_chipers() -> *mut libc::c_char {
    let ciphers_string = available_ciphers().join(",");
    let ciphers_c_string = CString::new(ciphers_string).unwrap_or_default();

    ciphers_c_string.into_raw()
}

/// Deallocates a CString returned by the Mullvad API client.
///
/// # Safety
///
/// `cstr_ptr` must be a pointer to a string allocated by another `mullvad_api` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_cstring_drop(cstr_ptr: *mut libc::c_char) {
    // SAFETY: caller guarantees that `cstr_ptr` is a valid C string pointer
    let _ = unsafe { CString::from_raw(cstr_ptr) };
}
