use std::ffi::c_void;

use mullvad_api::proxy::{ApiConnectionMode, ConnectionModeProvider};

extern "C" {
    pub fn connection_mode_provider_initial(rawPointer: *const c_void);
}

pub struct SwiftConnectionModeProvider {}

impl ConnectionModeProvider for SwiftConnectionModeProvider {
    fn initial(&self) -> ApiConnectionMode {
        unsafe {
            connection_mode_provider_initial(std::ptr::null());
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

impl SwiftConnectionModeProvider {
    pub fn new() -> Self {
        Self {}
    }
}
