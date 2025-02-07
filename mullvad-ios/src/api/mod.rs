use std::{ffi::CStr, ptr::null_mut, sync::Arc};

use mullvad_api::{
    proxy::{ApiConnectionMode, StaticConnectionModeProvider}, rest::MullvadRestHandle, ApiEndpoint, Runtime
};

mod api;
mod cancellation;
mod completion;
mod response;

#[repr(C)]
pub struct SwiftApiContext(*const ApiContext);
impl SwiftApiContext {
    pub fn new(context: ApiContext) -> SwiftApiContext {
        SwiftApiContext(Arc::into_raw(Arc::new(context)))
    }

    pub unsafe fn to_rust_context(self) -> Arc<ApiContext> {
        Arc::increment_strong_count(self.0);
        Arc::from_raw(self.0)
    }
}

pub struct ApiContext {
    _api_client: Runtime,
    rest_client: MullvadRestHandle,
}
impl ApiContext {
    pub fn rest_handle(&self) -> MullvadRestHandle {
        self.rest_client.clone()
    }
}

#[no_mangle]
pub extern "C" fn mullvad_api_init_new(host: *const u8, address: *const u8) -> SwiftApiContext {
    let host = unsafe { CStr::from_ptr(host.cast()) };
    let address = unsafe { CStr::from_ptr(address.cast()) };

    let Ok(host) = host.to_str() else {
        return SwiftApiContext(null_mut());
    };

    let Ok(address) = address.to_str() else {
        return SwiftApiContext(null_mut());
    };

    let endpoint = ApiEndpoint {
        host: Some(String::from(host)),
        address: Some(address.parse().unwrap()),
    };

    let tokio_handle = match crate::mullvad_ios_runtime() {
        Ok(tokio_handle) => tokio_handle,
        Err(err) => {
            log::error!("Failed to obtain a handle to a tokio runtime: {err}");
            return SwiftApiContext(null_mut());
        }
    };

    let api_context = tokio_handle.clone().block_on(async move {
        // It is imperative that the REST runtime is created within an async context, otherwise
        // ApiAvailability panics.
        let api_client = mullvad_api::Runtime::new(tokio_handle, &endpoint);
        let rest_client = api_client
            .mullvad_rest_handle(StaticConnectionModeProvider::new(ApiConnectionMode::Direct));

        ApiContext {
            _api_client: api_client,
            rest_client,
        }
    });

    SwiftApiContext::new(api_context)
}

