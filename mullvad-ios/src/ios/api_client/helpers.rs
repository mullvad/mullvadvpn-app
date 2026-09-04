use std::{
    ffi::CString,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use shadowsocks::crypto::available_ciphers;
use talpid_types::net::proxy::{Shadowsocks, ShadowsocksCipher, Socks5Remote, SocksAuth};

/// Constructs a new IP address from a pointer containing bytes representing an IP address.
///
/// SAFETY: `addr` pointer must be non-null, aligned, and point to at least addr_len bytes
pub(crate) fn parse_ip_addr(addr: &[u8]) -> Option<IpAddr> {
    match addr.len() {
        4 => Some(Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]).into()),
        16 => {
            let mut addr_arr = [0u8; 16];
            addr_arr.as_mut_slice().copy_from_slice(addr);

            Some(Ipv6Addr::from(addr_arr).into())
        }
        anything_else => {
            log::error!("Bad IP address length {anything_else}");
            None
        }
    }
}

#[derive(uniffi::Object)]
pub struct ShadowsocksWrapper(pub Shadowsocks);
/// Converts parameters into a boxed `Shadowsocks` configuration that is safe
/// to send across the FFI boundary
#[uniffi::export]
pub fn new_shadowsocks_access_method_setting(
    address: Vec<u8>,
    port: u16,
    password: String,
    cipher: String,
) -> Option<Arc<ShadowsocksWrapper>> {
    let endpoint: SocketAddr = SocketAddr::new(parse_ip_addr(&address)?, port);

    let cipher = ShadowsocksCipher::new(&cipher).ok()?;

    let shadowsocks_configuration = Shadowsocks::new(endpoint, cipher, password);

    Some(Arc::new(ShadowsocksWrapper(shadowsocks_configuration)))
}

#[derive(uniffi::Object)]
pub struct Socks5RemoteWrapper(pub Socks5Remote);
/// Converts parameters into a boxed `Socks5Remote` configuration that is safe
///
/// to send across the FFI boundary
#[uniffi::export]
pub fn new_socks5_access_method_setting(
    address: Vec<u8>,
    port: u16,
    username: Option<String>,
    password: Option<String>,
) -> Option<Arc<Socks5RemoteWrapper>> {
    let endpoint: SocketAddr = SocketAddr::new(parse_ip_addr(&address)?, port);

    let auth = SocksAuth::new(username.unwrap_or_default(), password.unwrap_or_default()).ok();

    let socks5_configuration = Socks5Remote { endpoint, auth };
    Some(Arc::new(Socks5RemoteWrapper(socks5_configuration)))
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
