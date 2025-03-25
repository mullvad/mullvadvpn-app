use std::{
    ffi::{c_char, c_void, CStr},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    ptr,
};

use mullvad_api::proxy::{ApiConnectionMode, ProxyConfig};
use talpid_types::net::proxy::{self, Shadowsocks};

extern "C" {
    pub fn swift_get_shadowsocks_bridges(rawBridgeProvider: *const c_void) -> *const c_void;
}

#[repr(C)]
pub struct SwiftShadowsocksLoaderWrapper(*mut SwiftShadowsocksLoaderWrapperContext);
impl SwiftShadowsocksLoaderWrapper {
    pub fn new(context: SwiftShadowsocksLoaderWrapperContext) -> SwiftShadowsocksLoaderWrapper {
        SwiftShadowsocksLoaderWrapper(Box::into_raw(Box::new(context)))
    }

    pub unsafe fn into_rust_context(self) -> Box<SwiftShadowsocksLoaderWrapperContext> {
        Box::from_raw(self.0)
    }
}

unsafe impl Sync for SwiftShadowsocksLoaderWrapper {}
unsafe impl Send for SwiftShadowsocksLoaderWrapper {}

#[derive(Debug)]
pub struct SwiftShadowsocksLoaderWrapperContext {
    shadowsocks_loader: *const c_void,
}

unsafe impl Sync for SwiftShadowsocksLoaderWrapperContext {}
unsafe impl Send for SwiftShadowsocksLoaderWrapperContext {}

//TODO: Write safety comments
impl SwiftShadowsocksLoaderWrapperContext {
    pub fn get_bridges(&self) -> Option<Shadowsocks> {
        let raw_configuration = unsafe { swift_get_shadowsocks_bridges(self.shadowsocks_loader) };
        if raw_configuration.is_null() {
            return None;
        }
        let bridges: Shadowsocks = unsafe { *Box::from_raw(raw_configuration as *mut _) };
        Some(bridges)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_swift_shadowsocks_loader_wrapper(
    shadowsocks_loader: *const c_void,
) -> SwiftShadowsocksLoaderWrapper {
    let context = SwiftShadowsocksLoaderWrapperContext { shadowsocks_loader };
    SwiftShadowsocksLoaderWrapper::new(context)
}

#[no_mangle]
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

    return Box::into_raw(Box::new(shadowsocks_configuration)) as *mut c_void;
}

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

unsafe fn convert_c_string(c_str: *const c_char) -> String {
    // SAFETY: domain_name points to a valid region of memory and contains a null terminator.
    let str = unsafe { CStr::from_ptr(c_str) };
    return String::from_utf8_lossy(str.to_bytes()).into_owned();
}
