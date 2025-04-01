use std::{ffi::c_char, future::Future, sync::Arc};

use access_method_resolver::SwiftAccessMethodResolver;
use access_method_settings::SwiftAccessMethodSettingsWrapper;
use helpers::convert_c_string;
use mullvad_api::{
    access_mode::{AccessModeSelector, AccessModeSelectorHandle},
    rest::{self, MullvadRestHandle},
    ApiEndpoint, Runtime,
};
use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState;
use mullvad_types::access_method::{Id, Settings};
use response::SwiftMullvadApiResponse;
use retry_strategy::RetryStrategy;
use shadowsocks_loader::SwiftShadowsocksLoaderWrapper;
use talpid_future::retry::retry_future;

mod access_method_resolver;
mod access_method_settings;
mod api;
mod cancellation;
mod completion;
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
    access_mode_handler: AccessModeSelectorHandle,
}
impl ApiContext {
    pub fn rest_handle(&self) -> MullvadRestHandle {
        self.rest_client.clone()
    }

    pub fn use_access_method(&self, id: Id) {
        _ = self.access_mode_handler.use_access_method(id);
    }

    pub fn update_access_methods(&self, access_methods: Settings) {
        _ = self
            .access_mode_handler
            .update_access_methods(access_methods)
    }
}

#[no_mangle]
pub unsafe extern "C" fn mullvad_api_update_access_methods(
    api_context: SwiftApiContext,
    settings_wrapper: SwiftAccessMethodSettingsWrapper,
) {
    let access_methods = settings_wrapper.into_rust_context().settings;
    api_context
        .into_rust_context()
        .update_access_methods(access_methods);
}

#[no_mangle]
pub unsafe extern "C" fn mullvad_api_use_access_method(
    api_context: SwiftApiContext,
    access_method_id: *const c_char,
) {
    let id = match Id::from_string(unsafe { convert_c_string(access_method_id) }) {
        Some(id) => id,
        None => return,
    };

    unsafe { api_context.into_rust_context().use_access_method(id) };
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
    host: *const c_char,
    address: *const c_char,
    domain: *const c_char,
    bridge_provider: SwiftShadowsocksLoaderWrapper,
    settings_provider: SwiftAccessMethodSettingsWrapper,
) -> SwiftApiContext {
    let host = unsafe { convert_c_string(host) };
    let address = unsafe { convert_c_string(address) };
    let domain = unsafe { convert_c_string(domain) };

    let endpoint = ApiEndpoint {
        host: Some(host),
        address: Some(address.parse().unwrap()),
    };

    let tokio_handle = crate::mullvad_ios_runtime().unwrap();

    let settings_context = unsafe { settings_provider.into_rust_context() };
    let access_method_settings = settings_context.convert_access_method().unwrap();
    let encrypted_dns_proxy_state = EncryptedDnsProxyState::default();

    let method_resolver = unsafe {
        SwiftAccessMethodResolver::new(
            endpoint.clone(),
            domain,
            encrypted_dns_proxy_state,
            *bridge_provider.into_rust_context(),
        )
    };
    // TODO: Handle #[cfg(feature = "api-override")]

    let api_context = tokio_handle.clone().block_on(async move {
        // It is imperative that the REST runtime is created within an async context, otherwise
        // ApiAvailability panics.

        let (access_mode_handler, access_mode_provider) =
            AccessModeSelector::spawn(method_resolver, access_method_settings)
                .await
                .expect("no errors here, move along");

        // TODO: Send the `access_mode_handler` back to swift to invoke update on access methods
        // TODO: Call `use_access_method` when the user manually switches access methods
        // TODO: Call `update_access_methods` when the user changes a method configuration

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
            access_mode_handler,
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
