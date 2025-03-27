use std::{ffi::CStr, ops::Deref, sync::Arc};

use access_method_resolver::SwiftAccessMethodResolver;
use access_method_settings::SwiftAccessMethodSettingsWrapper;
use connection_mode_provider::{connection_mode_provider_rotate, SwiftConnectionModeProvider};
use mullvad_api::{
    access_mode::{AccessModeSelector, Error},
    proxy::{ApiConnectionMode, StaticConnectionModeProvider},
    rest::{self, MullvadRestHandle},
    ApiEndpoint, Runtime,
};
use shadowsocks_loader::SwiftShadowsocksLoaderWrapper;

mod access_method_resolver;
mod access_method_settings;
mod account;
mod api;
mod cancellation;
mod completion;
mod problem_report;
mod connection_mode_provider;
mod helpers;
mod response;
mod retry_strategy;
mod shadowsocks_loader;

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
pub extern "C" fn mullvad_api_init_new(
    host: *const u8,
    address: *const u8,
    bridge_provider: SwiftShadowsocksLoaderWrapper,
    settings_provider: SwiftAccessMethodSettingsWrapper,
    provider: SwiftConnectionModeProvider,
) -> SwiftApiContext {
    let host = unsafe { CStr::from_ptr(host.cast()) };
    let address = unsafe { CStr::from_ptr(address.cast()) };

    let host = host.to_str().unwrap();
    let address = address.to_str().unwrap();

    let endpoint = ApiEndpoint {
        host: Some(String::from(host)),
        address: Some(address.parse().unwrap()),
    };

    let tokio_handle = crate::mullvad_ios_runtime().unwrap();

    let connection_mode_provider_context = unsafe { provider.into_rust_context() };
    let settings_context = unsafe { settings_provider.into_rust_context() };
    let access_method_settings = settings_context.convert_access_method().unwrap();

    let method_resolver = unsafe {
        SwiftAccessMethodResolver::new(endpoint.clone(), *bridge_provider.into_rust_context())
    };
    println!(
        "{:?}, {:?}, {:?}",
        method_resolver, settings_context, access_method_settings
    );
    // TODO: Use the method_resolver in the AccessModeSelector::spawn call
    // TODO: Bridge settings.api_access_methods
    // TODO: Handle #[cfg(feature = "api-override")]
    // TODO: Handle access_method_event_sender, used for "sending", should we just remove that parameter from iOS?
    tokio_handle.spawn(connection_mode_provider_context.spawn_rotator());

    let api_context = tokio_handle.clone().block_on(async move {
        // It is imperative that the REST runtime is created within an async context, otherwise
        // ApiAvailability panics.

        let (access_mode_handler, access_mode_provider) =
            AccessModeSelector::spawn(method_resolver, access_method_settings)
                .await
                .expect("no errors here, move along");

        // TODO: Should this be sent back to swift to invoke when the user changes access methods?

        let api_client = mullvad_api::Runtime::new(tokio_handle, &endpoint);
        let rest_client = api_client.mullvad_rest_handle(access_mode_provider);

        // tokio::spawn(async move {
        //     let _ = access_mode_handler
        //         .update_access_methods(access_method_settings)
        //         .await;
        // });

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
    let response = retry_request(retry_strategy, future_factory).await?;
    SwiftMullvadApiResponse::with_body(response).await
}

async fn do_request_with_empty_body<F, T>(
    retry_strategy: RetryStrategy,
    future_factory: F,
) -> Result<SwiftMullvadApiResponse, rest::Error>
where
    F: Fn() -> T,
    T: Future<Output = Result<(), rest::Error>>,
{
    retry_request(retry_strategy, future_factory).await?;
    Ok(SwiftMullvadApiResponse::ok())
}

async fn retry_request<F, T, U>(
    retry_strategy: RetryStrategy,
    future_factory: F,
) -> Result<U, rest::Error>
where
    F: Fn() -> T,
    T: Future<Output = Result<U, rest::Error>>,
{
    let should_retry = |result: &Result<_, rest::Error>| match result {
        Err(err) => err.is_network_error(),
        Ok(_) => false,
    };

    retry_future(future_factory, should_retry, retry_strategy.delays()).await
}
