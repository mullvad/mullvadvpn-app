use libc::c_char;
use std::{ffi::c_void, net::SocketAddr};

use super::get_string;

unsafe extern "C" {
    /// Return the latest available endpoint, or a default one if none are cached
    ///
    /// # SAFETY
    /// `rawAddressCacheProvider` **must** be provided by a call to `init_swift_address_cache_wrapper`
    /// It is okay to persist it, and use it accross multiple threads.
    pub fn swift_get_cached_endpoint(
        rawAddressCacheProvider: *const c_void,
    ) -> LateStringDeallocator;
}

#[derive(Debug)]
#[repr(C)]
pub struct SwiftAddressCacheWrapper(SwiftAddressCacheProviderContext);

impl SwiftAddressCacheWrapper {
    pub fn new(context: SwiftAddressCacheProviderContext) -> SwiftAddressCacheWrapper {
        SwiftAddressCacheWrapper(context)
    }

    pub fn get_addrs(&self) -> SocketAddr {
        self.context_ref().get_addrs()
    }

    fn context_ref(&self) -> &SwiftAddressCacheProviderContext {
        &self.0
    }
}

// SAFETY: The `address_cache` inside the `SwiftAddressCacheProviderContext`'s context is guaranteed to be thread safe
unsafe impl Sync for SwiftAddressCacheProviderContext {}
// SAFETY: The `address_cache` inside the `SwiftAddressCacheProviderContext`'s context is guaranteed to be Sendable
unsafe impl Send for SwiftAddressCacheProviderContext {}

#[derive(Debug)]
#[repr(C)]
pub struct SwiftAddressCacheProviderContext {
    address_cache: *const c_void,
}

/// A struct used to deallocate a pointer to a C String later than when the pointer's control is relinquished from Swift.
/// Use the `deallocate_ptr` function on `ptr` to call the custom deallocator provided by Swift.
#[repr(C)]
pub struct LateStringDeallocator {
    pub(crate) ptr: *const c_char,
    deallocate_ptr: unsafe extern "C" fn(*const c_char),
}

impl SwiftAddressCacheProviderContext {
    pub fn get_addrs(&self) -> SocketAddr {
        // SAFETY: See notice for `swift_get_cached_endpoint`
        let deallocator = unsafe { swift_get_cached_endpoint(self.address_cache) };

        // SAFETY: The pointer contained in the late deallocator returned by `swift_get_cached_endpoint`
        // is guaranteed to point to a valid UTF-8 String
        // It is also guaranteed to be a valid representation of either an IPv4 or IPv6 address
        let cached_address = unsafe { get_string(deallocator.ptr) }
            .parse()
            .expect("Invalid socket address in cache");

        // SAFETY: The pointer in `deallocator.ptr` must not be used after `deallocate_ptr` has been called.
        // `deallocate_ptr` must be called only once
        unsafe { (deallocator.deallocate_ptr)(deallocator.ptr) };
        cached_address
    }
}

/// Called by the Swift side in order to provide an object to rust that provides API addresses in a UTF-8 string form
///
/// # SAFETY
/// `address_cache` **must be** pointing to a valid instance of a `DefaultAddressCacheProvider`
/// That instance's lifetime has to be equivalent to a `'static` lifetime in Rust
/// This function does not take ownership of `address_cache`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_swift_address_cache_wrapper(
    address_cache: *const c_void,
) -> SwiftAddressCacheWrapper {
    let context = SwiftAddressCacheProviderContext { address_cache };
    SwiftAddressCacheWrapper::new(context)
}
