use std::ffi::c_void;
use talpid_types::net::proxy::Shadowsocks;

extern "C" {
    /// Creates a `Shadowsocks` configuration.
    ///
    /// # SAFETY
    /// `rawBridgeProvider` **must** be provided by a call to `init_swift_shadowsocks_loader_wrapper`
    /// It is okay to persist it, and use it across multiple threads.
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

// SAFETY: The context stored inside `SwiftShadowsocksLoaderWrapper` points to an object that is guaranteed to be thread safe
unsafe impl Sync for SwiftShadowsocksLoaderWrapper {}
unsafe impl Send for SwiftShadowsocksLoaderWrapper {}

#[derive(Debug)]
pub struct SwiftShadowsocksLoaderWrapperContext {
    shadowsocks_loader: *const c_void,
}

unsafe impl Sync for SwiftShadowsocksLoaderWrapperContext {}
unsafe impl Send for SwiftShadowsocksLoaderWrapperContext {}

impl SwiftShadowsocksLoaderWrapperContext {
    pub fn get_bridges(&self) -> Option<Shadowsocks> {
        // SAFETY: See notice for `swift_get_shadowsocks_bridges`
        let raw_configuration = unsafe { swift_get_shadowsocks_bridges(self.shadowsocks_loader) };
        if raw_configuration.is_null() {
            return None;
        }
        // SAFETY: The pointer returned by `swift_get_shadowsocks_bridges` is guaranteed
        // to point to a valid `Shadowsocks` configuration
        let bridges: Shadowsocks = unsafe { *Box::from_raw(raw_configuration as *mut _) };
        Some(bridges)
    }
}

/// Called by the Swift side in order to provide an object to rust that can create
/// Shadowsocks configurations
///
/// # SAFETY
/// `shadowsocks_loader` **must be** pointing to a valid instance of a `SwiftShadowsocksBridgeProvider`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_swift_shadowsocks_loader_wrapper(
    shadowsocks_loader: *const c_void,
) -> SwiftShadowsocksLoaderWrapper {
    let context = SwiftShadowsocksLoaderWrapperContext { shadowsocks_loader };
    SwiftShadowsocksLoaderWrapper::new(context)
}
