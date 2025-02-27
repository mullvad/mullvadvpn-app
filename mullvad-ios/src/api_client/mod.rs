use std::{ffi::CStr, future::Future, sync::Arc};

use mullvad_api::{
    proxy::{ApiConnectionMode, StaticConnectionModeProvider},
    rest::{self, MullvadRestHandle},
    ApiEndpoint, Runtime,
};
use response::SwiftMullvadApiResponse;
use retry_strategy::RetryStrategy;
use talpid_future::retry::retry_future;

mod api;
mod cancellation;
mod completion;
mod mock;
mod response;
mod retry_strategy;

#[repr(C)]
pub struct SwiftApiContext(*const ApiContext);
impl SwiftApiContext {
    pub fn new(context: ApiContext) -> SwiftApiContext {
        SwiftApiContext(Arc::into_raw(Arc::new(context)))
    }

    pub unsafe fn into_rust_context(self) -> Arc<ApiContext> {
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

/// # Safety
///
/// `host` must be a pointer to a null terminated string representing a hostname for Mullvad API host.
/// This hostname will be used for TLS validation but not used for domain name resolution.
///
/// `address` must be a pointer to a null terminated string representing a socket address through which
/// the Mullvad API can be reached directly.
///
/// If a context cannot be constructed this function will panic since the call site would not be able
/// to proceed in a meaningful way anyway.
///
/// This function is safe.
#[no_mangle]
pub extern "C" fn mullvad_api_init_new_tls_disabled(
    host: *const u8,
    address: *const u8,
) -> SwiftApiContext {
    mullvad_api_init_inner(host, address, true)
}

/// # Safety
///
/// `host` must be a pointer to a null terminated string representing a hostname for Mullvad API host.
/// This hostname will be used for TLS validation but not used for domain name resolution.
///
/// `address` must be a pointer to a null terminated string representing a socket address through which
/// the Mullvad API can be reached directly.
///
/// If a context cannot be constructed this function will panic since the call site would not be able
/// to proceed in a meaningful way anyway.
///
/// This function is safe.
#[no_mangle]
pub extern "C" fn mullvad_api_init_new(host: *const u8, address: *const u8) -> SwiftApiContext {
    mullvad_api_init_inner(host, address, false)
}

fn mullvad_api_init_inner(
    host: *const u8,
    address: *const u8,
    disable_tls: bool,
) -> SwiftApiContext {
    let host = unsafe { CStr::from_ptr(host.cast()) };
    let address = unsafe { CStr::from_ptr(address.cast()) };

    let host = host.to_str().unwrap();
    let address = address.to_str().unwrap();

    let endpoint = ApiEndpoint {
        host: Some(String::from(host)),
        address: Some(address.parse().unwrap()),
        #[cfg(feature = "api-override")]
        disable_tls,
        #[cfg(feature = "api-override")]
        force_direct: false,
    };

    let tokio_handle = crate::mullvad_ios_runtime().unwrap();

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

async fn do_request<F, T>(
    retry_strategy: RetryStrategy,
    future_factory: F,
) -> Result<SwiftMullvadApiResponse, rest::Error>
where
    F: Fn() -> T,
    T: Future<Output = Result<rest::Response<hyper::body::Incoming>, rest::Error>>,
{
    let should_retry = |result: &Result<_, rest::Error>| match result {
        Err(err) => err.is_network_error(),
        Ok(_) => false,
    };

    let response = retry_future(future_factory, should_retry, retry_strategy.delays()).await?;

    SwiftMullvadApiResponse::with_body(response).await
}
