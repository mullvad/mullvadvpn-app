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
mod account;
mod api;
mod cancellation;
mod completion;
pub(super) mod helpers;
mod problem_report;
mod response;
mod retry_strategy;
mod shadowsocks_loader;

#[repr(C)]
pub struct SwiftApiContext(*const ApiContext);
impl SwiftApiContext {
    pub fn new(context: ApiContext) -> SwiftApiContext {
        SwiftApiContext(Arc::into_raw(Arc::new(context)))
    }

    /// Extracts an `ApiContext` from `self`
    ///
    /// # Safety
    ///
    /// The `ApiContext` extracted is meant to live as long as the process it's used in.
    /// This should always be safe to call.
    pub unsafe fn rust_context(self) -> Arc<ApiContext> {
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

    /// Sets the access method referenced by `id` as currently in use.
    ///
    /// This function will block the current thread until it is complete,
    /// make sure to not call this from a UI Thread if possible.
    pub fn use_access_method(&self, id: Id) {
        _ = self
            ._api_client
            .handle()
            .block_on(async { self.access_mode_handler.use_access_method(id).await });
    }

    /// Replaces the current set of access methods with `access_methods.
    ///
    /// This function will block the current thread until it is complete,
    /// make sure to not call this from a UI Thread if possible.
    pub fn update_access_methods(&self, access_methods: Settings) {
        _ = self._api_client.handle().block_on(async {
            self.access_mode_handler
                .update_access_methods(access_methods)
                .await
        });
    }
}

/// Called by Swift to set the available access methods
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_update_access_methods(
    api_context: SwiftApiContext,
    settings_wrapper: SwiftAccessMethodSettingsWrapper,
) {
    let access_methods = settings_wrapper.into_rust_context().settings;
    api_context
        .rust_context()
        .update_access_methods(access_methods);
}

/// Called by Swift to update the currently used access methods
///
/// # SAFETY
/// `access_method_id` must point to a null terminated string in a UUID format
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_use_access_method(
    api_context: SwiftApiContext,
    access_method_id: *const c_char,
) {
    // SAFETY: See notes for `rust_context`
    let api_context = unsafe { api_context.rust_context() };
    // SAFETY: See Safety notes for `convert_c_string`
    let id = unsafe { convert_c_string(access_method_id) };

    let Some(id) = Id::from_string(id) else {
        return;
    };
    api_context.use_access_method(id);
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
#[unsafe(no_mangle)]
pub extern "C" fn mullvad_api_init_new(
    host: *const c_char,
    address: *const c_char,
    domain: *const c_char,
    bridge_provider: SwiftShadowsocksLoaderWrapper,
    settings_provider: SwiftAccessMethodSettingsWrapper,
) -> SwiftApiContext {
    // Safety: See notes for `convert_c_string`
    let (host, address, domain) = unsafe {
        (
            convert_c_string(host),
            convert_c_string(address),
            convert_c_string(domain),
        )
    };

    // The iOS client provides a different default endpoint based on its configuration
    // Debug and Release builds use the standard endpoints
    // Staging builds will use the staging endpoint
    let endpoint = ApiEndpoint {
        host: Some(host),
        address: Some(address.parse().unwrap()),
        #[cfg(feature = "api-override")]
        disable_tls: false,
        #[cfg(feature = "api-override")]
        force_direct: true,
    };

    let tokio_handle = crate::mullvad_ios_runtime().unwrap();

    // SAFETY: See notes for `into_rust_context`
    let settings_context = unsafe { settings_provider.into_rust_context() };
    let access_method_settings = settings_context.convert_access_method().unwrap();
    let encrypted_dns_proxy_state = EncryptedDnsProxyState::default();

    // TODO: Add a wrapper around the iOS AddressCache in SwiftAccessMethodResolver
    // So that it can be used in the `default_connection_mode` implementation
    let method_resolver = SwiftAccessMethodResolver::new(
        endpoint.clone(),
        domain,
        encrypted_dns_proxy_state,
        bridge_provider,
    );

    let api_context = tokio_handle.clone().block_on(async move {
        let (access_mode_handler, access_mode_provider) = AccessModeSelector::spawn(
            method_resolver,
            access_method_settings,
            #[cfg(feature = "api-override")]
            endpoint.clone(),
        )
        .await
        .expect("Could now spawn AccessModeSelector");

        // It is imperative that the REST runtime is created within an async context, otherwise
        // ApiAvailability panics.
        let api_client = mullvad_api::Runtime::new(tokio_handle, &endpoint);
        let rest_client = api_client.mullvad_rest_handle(access_mode_provider);

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
