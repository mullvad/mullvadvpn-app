use std::{ffi::c_char, ffi::c_void, future::Future, sync::Arc};

use crate::get_string;
use access_method_resolver::{IOSAddressCacheBacking, SwiftAccessMethodResolver};
use access_method_settings::SwiftAccessMethodSettingsWrapper;
use address_cache_provider::SwiftAddressCacheWrapper;
use futures::{
    StreamExt,
    channel::{mpsc, oneshot},
};
use mullvad_api::{
    ApiEndpoint, Runtime,
    access_mode::{AccessMethodEvent, AccessModeSelector, AccessModeSelectorHandle},
    rest::{self, MullvadRestHandle},
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
mod address_cache_provider;
mod api;
mod cancellation;
mod completion;
mod device;
pub(super) mod helpers;
mod mock;
mod problem_report;
mod response;
mod retry_strategy;
mod shadowsocks_loader;
mod storekit;

#[repr(C)]
pub struct SwiftApiContext(*const ApiContext);
impl SwiftApiContext {
    pub fn new(context: ApiContext) -> SwiftApiContext {
        SwiftApiContext(Arc::into_raw(Arc::new(context)))
    }

    /// Extracts an `ApiContext` from `self`
    ///
    /// The `ApiContext` extracted is meant to live as long as the process it's used in.
    pub fn rust_context(self) -> Arc<ApiContext> {
        // SAFETY: This will never be deallocated
        unsafe {
            Arc::increment_strong_count(self.0);
            Arc::from_raw(self.0)
        }
    }
}

pub struct ApiContext {
    api_client: Runtime,
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
            .api_client
            .handle()
            .block_on(async { self.access_mode_handler.use_access_method(id).await });
    }

    /// Replaces the current set of access methods with `access_methods.
    ///
    /// This function will block the current thread until it is complete,
    /// make sure to not call this from a UI Thread if possible.
    pub fn update_access_methods(&self, access_methods: Settings) {
        _ = self.api_client.handle().block_on(async {
            self.access_mode_handler
                .update_access_methods(access_methods)
                .await
        });
    }
}

/// An opaque pointer that exists only to be passed from the caller to a callback through the ABI
struct ForeignPtr {
    ptr: *const c_void,
}
/// allow this to be passed across thread boundaries
unsafe impl Send for ForeignPtr {}

/// Called by Swift to set the available access methods
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_update_access_methods(
    api_context: SwiftApiContext,
    settings_wrapper: SwiftAccessMethodSettingsWrapper,
) {
    let access_methods = unsafe { settings_wrapper.into_rust_context().settings };
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
    let api_context = api_context.rust_context();
    // SAFETY: See Safety notes for `get_string`
    let id = unsafe { get_string(access_method_id) };

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
#[cfg(feature = "api-override")]
#[unsafe(no_mangle)]
pub extern "C" fn mullvad_api_init_new_tls_disabled(
    host: *const c_char,
    address: *const c_char,
    domain: *const c_char,
    bridge_provider: SwiftShadowsocksLoaderWrapper,
    settings_provider: SwiftAccessMethodSettingsWrapper,
    address_cache: SwiftAddressCacheWrapper,
    access_method_change_callback: Option<unsafe extern "C" fn(*const c_void, *const u8)>,
    access_method_change_context: *const c_void,
) -> SwiftApiContext {
    mullvad_api_init_inner(
        host,
        address,
        domain,
        true,
        bridge_provider,
        settings_provider,
        address_cache,
        access_method_change_callback,
        access_method_change_context,
    )
}

/// # Safety
///
/// `host` must be a pointer to a null terminated string representing a hostname for Mullvad API host.
/// This hostname will be used for TLS validation but not used for domain name resolution.
///
/// `address` must be a pointer to a null terminated string representing a socket address through which
/// the Mullvad API can be reached directly.
///
/// address_method_change_callback is a function with the C calling convention which will be called
/// whenever the access method changes with a user-specified opaque pointer and a pointer to the bytes
/// of the access method's UUID. Note that this callback must remain valid for the lifetime of the
/// program.
///
/// access_method_change_context is the pointer passed verbatim to the callback. It is not dereferenced
/// by the Rust code, but remains opaque.
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
    address_cache: SwiftAddressCacheWrapper,
    access_method_change_callback: Option<unsafe extern "C" fn(*const c_void, *const u8)>,
    access_method_change_context: *const c_void,
) -> SwiftApiContext {
    #[cfg(feature = "api-override")]
    return mullvad_api_init_inner(
        host,
        address,
        domain,
        false,
        bridge_provider,
        settings_provider,
        address_cache,
        access_method_change_callback,
        access_method_change_context,
    );
    #[cfg(not(feature = "api-override"))]
    mullvad_api_init_inner(
        host,
        address,
        domain,
        bridge_provider,
        settings_provider,
        address_cache,
        access_method_change_callback,
        access_method_change_context,
    )
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
pub extern "C" fn mullvad_api_init_inner(
    host: *const c_char,
    address: *const c_char,
    domain: *const c_char,
    #[cfg(feature = "api-override")] disable_tls: bool,
    bridge_provider: SwiftShadowsocksLoaderWrapper,
    settings_provider: SwiftAccessMethodSettingsWrapper,
    address_cache: SwiftAddressCacheWrapper,
    access_method_change_callback: Option<unsafe extern "C" fn(*const c_void, *const u8)>,
    access_method_change_context: *const c_void,
) -> SwiftApiContext {
    // Safety: See notes for `get_string`
    let (host, address, domain) =
        unsafe { (get_string(host), get_string(address), get_string(domain)) };

    // The iOS client provides a different default endpoint based on its configuration
    // Debug and Release builds use the standard endpoints
    // Staging builds will use the staging endpoint
    let endpoint = ApiEndpoint {
        host: Some(host),
        address: Some(address.parse().unwrap()),
        #[cfg(feature = "api-override")]
        disable_tls,
        #[cfg(feature = "api-override")]
        force_direct: false,
    };

    let tokio_handle = crate::mullvad_ios_runtime().unwrap();

    // SAFETY: See notes for `into_rust_context`
    let settings_context = unsafe { settings_provider.into_rust_context() };
    let access_method_settings = settings_context.convert_access_method().unwrap();
    let encrypted_dns_proxy_state = EncryptedDnsProxyState::default();

    let method_resolver = SwiftAccessMethodResolver::new(
        endpoint.clone(),
        domain,
        encrypted_dns_proxy_state,
        bridge_provider,
        address_cache,
    );

    let access_method_change_ctx: ForeignPtr = ForeignPtr {
        ptr: access_method_change_context,
    };
    let api_context = tokio_handle.clone().block_on(async move {
        let (tx, mut rx) = mpsc::unbounded::<(AccessMethodEvent, oneshot::Sender<()>)>();
        let (access_mode_handler, access_mode_provider) = AccessModeSelector::spawn(
            method_resolver,
            access_method_settings,
            #[cfg(feature = "api-override")]
            endpoint.clone(),
            tx,
        )
        .await
        .expect("Could now spawn AccessModeSelector");

        // SAFETY: The callback is expected to be called from the Swift side
        if let Some(callback) = access_method_change_callback {
            tokio::spawn(async move {
                let access_method_change_ctx = access_method_change_ctx;
                while let Some((event, _sender)) = rx.next().await {
                    let AccessMethodEvent::New {
                        setting,
                        connection_mode: _,
                        endpoint: _,
                    } = event
                    else {
                        continue;
                    };
                    let uuid = setting.get_id();
                    let uuid_bytes = uuid.as_bytes();
                    // SAFETY: The callback is expected to be safe to call
                    unsafe { callback(access_method_change_ctx.ptr, uuid_bytes.as_ptr()) };
                }
            });
        }

        // It is imperative that the REST runtime is created within an async context, otherwise
        // ApiAvailability panics.

        let api_client = mullvad_api::Runtime::with_cache_backing(tokio_handle, &endpoint, Arc::new(IOSAddressCacheBacking {})).await;
        let rest_client = api_client.mullvad_rest_handle(access_mode_provider);

        ApiContext {
            api_client,
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
