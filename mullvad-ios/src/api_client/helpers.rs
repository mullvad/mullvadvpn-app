use std::{
    ffi::{c_char, c_void, CStr},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

use mullvad_types::access_method::AccessMethodSetting;
use talpid_types::net::proxy::{Shadowsocks, Socks5Remote, SocksAuth};

/// Constructs a new IP address from a pointer containing bytes representing an IP address.
///
/// SAFETY: `addr` must be a pointer to at least `addr_len` bytes.
pub unsafe fn parse_ip_addr(addr: *const u8, addr_len: usize) -> Option<IpAddr> {
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
        anything_else => {
            log::error!("Bad IP address length {anything_else}");
            None
        }
    }
}

/// Converts a pointer to a C style string into an owned Rust `String`
///
/// # SAFETY
/// `c_str` must point to a valid, null terminated C string.
pub unsafe fn convert_c_string(c_str: *const c_char) -> String {
    // SAFETY: c_str points to a valid region of memory and contains a null terminator.
    let str = unsafe { CStr::from_ptr(c_str) };
    String::from_utf8_lossy(str.to_bytes()).into_owned()
}

/// Converts parameters into a boxed `Shadowsocks` configuration that is safe
/// to send across the FFI boundary
///
/// # SAFETY
/// `address` must be a pointer to at least `address_len` bytes.
/// `c_password` and `c_cipher` must be pointers to null terminated strings
#[unsafe(no_mangle)]
pub unsafe extern "C" fn convert_shadowsocks(
    address: *const u8,
    address_len: usize,
    port: u16,
    c_password: *const c_char,
    c_cipher: *const c_char,
) -> *const c_void {
    let endpoint: SocketAddr = if let Some(ip_address) = parse_ip_addr(address, address_len) {
        SocketAddr::new(ip_address, port)
    } else {
        return std::ptr::null();
    };

    let password = convert_c_string(c_password);
    let cipher = convert_c_string(c_cipher);

    let shadowsocks_configuration = Shadowsocks {
        endpoint,
        password,
        cipher,
    };

    Box::into_raw(Box::new(shadowsocks_configuration)) as *mut c_void
}

#[repr(C)]
pub struct RustAccessMethodSettingVector {
    inner: *mut RustAccessMethodSettingVectorContext,
}

impl RustAccessMethodSettingVector {
    pub unsafe fn into_rust_context(self) -> Box<RustAccessMethodSettingVectorContext> {
        Box::from_raw(self.inner)
    }

    pub unsafe fn push(&mut self, setting: AccessMethodSetting) {
        if let Some(inner) = unsafe { self.inner.as_mut() } {
            inner.push(setting);
        }
    }
}

pub struct RustAccessMethodSettingVectorContext {
    pub vector: Vec<AccessMethodSetting>,
}

impl RustAccessMethodSettingVectorContext {
    pub unsafe fn push(&mut self, setting: AccessMethodSetting) {
        self.vector.push(setting);
    }
}

/// Creates a wrapper around a Rust `Vec` type that can be safely sent across the FFI boundary.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn access_method_settings_vector(
    capacity: usize,
) -> RustAccessMethodSettingVector {
    let vector: Vec<AccessMethodSetting> = Vec::with_capacity(capacity);
    let context = Box::new(RustAccessMethodSettingVectorContext { vector });
    RustAccessMethodSettingVector {
        inner: Box::into_raw(context),
    }
}

/// Adds an `AccessMethodSetting` to the inner vector contained in `vector_raw`
///
/// # SAFETY
/// `access_method` must be a pointer gotten through a call to `convert_builtin_access_method_setting`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vector_add_access_method_setting(
    vector_raw: RustAccessMethodSettingVector,
    access_method: *const c_void,
) {
    if !access_method.is_null() {
        // SAFETY: see `vector_add_access_method_setting`
        let access_method: AccessMethodSetting = unsafe { *Box::from_raw(access_method as *mut _) };
        let mut vector = vector_raw;
        vector.push(access_method);
    }
}

/// Converts parameters into a boxed `Socks5Remote` configuration that is safe
/// to send across the FFI boundary
///
/// # SAFETY
/// `address` must be a pointer to at least `address_len` bytes.
/// `c_username` and `c_password` must be a pointer to null terminated strings
#[unsafe(no_mangle)]
pub unsafe extern "C" fn convert_socks5(
    address: *const u8,
    address_len: usize,
    port: u16,
    c_username: *const c_char,
    c_password: *const c_char,
) -> *const c_void {
    let endpoint: SocketAddr = if let Some(ip_address) = parse_ip_addr(address, address_len) {
        SocketAddr::new(ip_address, port)
    } else {
        return std::ptr::null();
    };

    let mut auth: Option<SocksAuth> = None;
    if !c_username.is_null() && !c_password.is_null() {
        let username = convert_c_string(c_username);
        let password = convert_c_string(c_password);
        auth = match SocksAuth::new(username, password) {
            Ok(auth) => Some(auth),
            Err(_) => None,
        }
    }

    let socks5_configuration = Socks5Remote { endpoint, auth };
    Box::into_raw(Box::new(socks5_configuration)) as *mut c_void
}
