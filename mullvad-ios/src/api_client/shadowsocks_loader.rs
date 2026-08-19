use std::ffi::c_void;

#[derive(Debug)]
#[repr(C)]
pub struct SwiftShadowsocksLoaderWrapper(SwiftShadowsocksLoaderWrapperContext);
impl SwiftShadowsocksLoaderWrapper {
    pub fn new(context: SwiftShadowsocksLoaderWrapperContext) -> SwiftShadowsocksLoaderWrapper {
        SwiftShadowsocksLoaderWrapper(context)
    }
}

// SAFETY: The context stored inside `SwiftShadowsocksLoaderWrapper` points to an object that is guaranteed to be thread safe
unsafe impl Sync for SwiftShadowsocksLoaderWrapper {}
// SAFETY: The context stored inside `SwiftShadowsocksLoaderWrapper` points to an object that is guaranteed to be Sendable
unsafe impl Send for SwiftShadowsocksLoaderWrapper {}

#[derive(Debug)]
#[repr(C)]
pub struct SwiftShadowsocksLoaderWrapperContext {
    // This pointer is a reference to a Swift object, and is only ever read by Rust.
    // It is used to call that Swift object across the FFI
    shadowsocks_loader: *const c_void,
}

// SAFETY: `shadowsocks_loader` inside the `SwiftShadowsocksLoaderWrapperContext ` points to an object that is guaranteed to be thread safe
unsafe impl Sync for SwiftShadowsocksLoaderWrapperContext {}
// SAFETY: `shadowsocks_loader` inside the `SwiftShadowsocksLoaderWrapperContext ` points to an object that is guaranteed to be Sendable
unsafe impl Send for SwiftShadowsocksLoaderWrapperContext {}

impl SwiftShadowsocksLoaderWrapperContext {}

/// Called by the Swift side in order to provide an object to rust that can create
/// Shadowsocks configurations
///
/// # SAFETY
/// `shadowsocks_loader` **must be** pointing to a valid instance of a `SwiftShadowsocksBridgeProvider`
/// That instance's lifetime has to be equivalent to a `'static` lifetime in Rust
/// This function does not take ownership of `shadowsocks_loader`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_swift_shadowsocks_loader_wrapper(
    shadowsocks_loader: *const c_void,
) -> SwiftShadowsocksLoaderWrapper {
    let context = SwiftShadowsocksLoaderWrapperContext { shadowsocks_loader };
    SwiftShadowsocksLoaderWrapper::new(context)
}
