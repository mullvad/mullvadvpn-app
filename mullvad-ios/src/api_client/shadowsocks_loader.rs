use std::{
    ffi::{c_char, c_void, CStr},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    ptr,
};

use mullvad_api::proxy::{ApiConnectionMode, ProxyConfig};
use talpid_types::net::proxy::{self, Shadowsocks};

use super::helpers::{convert_c_string, parse_ip_addr};

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
