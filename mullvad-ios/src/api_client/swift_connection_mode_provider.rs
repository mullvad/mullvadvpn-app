use std::{ffi::c_void, ptr::null_mut, sync::Arc};

use mullvad_api::proxy::{ApiConnectionMode, ConnectionModeProvider};

extern "C" {
    pub fn connection_mode_provider_initial(rawPointer: *const c_void);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_connection_mode_provider(
    raw_provider: *const c_void,
) -> SwiftConnectionModeProvider {
    let context = SwiftConnectionModeProviderContext {
        provider: raw_provider,
    };
    SwiftConnectionModeProvider::new(context)
}

#[repr(C)]
pub struct SwiftConnectionModeProvider(*const SwiftConnectionModeProviderContext);
impl SwiftConnectionModeProvider {
    pub fn new(context: SwiftConnectionModeProviderContext) -> SwiftConnectionModeProvider {
        SwiftConnectionModeProvider(Arc::into_raw(Arc::new(context)))
    }

    pub unsafe fn into_rust_context(self) -> Arc<SwiftConnectionModeProviderContext> {
        Arc::increment_strong_count(self.0);
        Arc::from_raw(self.0)
    }
}

pub struct SwiftConnectionModeProviderContext {
    provider: *const c_void,
}

impl Copy for SwiftConnectionModeProviderContext {}

impl Clone for SwiftConnectionModeProviderContext {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl Send for SwiftConnectionModeProviderContext {}

impl ConnectionModeProvider for SwiftConnectionModeProviderContext {
    fn initial(&self) -> ApiConnectionMode {
        unsafe {
            connection_mode_provider_initial(self.provider);
        }
        ApiConnectionMode::Direct
    }

    fn rotate(&self) -> impl std::future::Future<Output = ()> + Send {
        futures::future::ready(())
    }

    fn receive(&mut self) -> impl std::future::Future<Output = Option<ApiConnectionMode>> + Send {
        // tokio::runtime::Handle::current().spawn_blocking(
        async { Some(ApiConnectionMode::Direct) }
    }
}
