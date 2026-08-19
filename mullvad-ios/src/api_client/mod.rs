use std::{
    ffi::{c_char, c_void},
    future::Future,
    net::SocketAddr,
    sync::Arc,
};

use crate::{
    api_client::{
        access_method_settings::SwiftAccessMethodSettingsContext, helpers::parse_ip_addr,
    },
    get_string,
};
use access_method_resolver::{IOSAddressCacheBacking, SwiftAccessMethodResolver};
use access_method_settings::SwiftAccessMethodSettingsWrapper;
use futures::{
    StreamExt,
    channel::{mpsc, oneshot},
};
use mullvad_api::{
    AddressCache, ApiEndpoint, ApiProxy, Runtime,
    access_mode::{AccessMethodEvent, AccessModeSelector, AccessModeSelectorHandle},
    rest::{self, MullvadRestHandle},
};
use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState;
use mullvad_types::access_method::{Id, Settings};
use response::SwiftMullvadApiResponse;
use retry_strategy::RetryStrategy;
use talpid_future::retry::retry_future;
use talpid_types::net::proxy::{Shadowsocks, ShadowsocksCipher};
use zerocopy::IntoBytes;

mod access_method_resolver;
mod access_method_settings;
mod account;
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
mod swift_data;

#[repr(C)]
#[derive(uniffi::Record)]
pub struct SwiftApiContext {
    ptr: u64,
}
impl SwiftApiContext {
    pub fn new(context: ApiContext) -> SwiftApiContext {
        SwiftApiContext {
            ptr: Arc::into_raw(Arc::new(context)) as u64,
        }
    }

    /// Extracts an `ApiContext` from `self`
    ///
    /// The `ApiContext` extracted is meant to live as long as the process it's used in.
    pub fn rust_context(self) -> Arc<ApiContext> {
        // SAFETY: This will never be deallocated
        unsafe {
            Arc::increment_strong_count(self.ptr as *const ApiContext);
            Arc::from_raw(self.ptr as *const ApiContext)
        }
    }
}

#[derive(uniffi::Record)]
pub struct UnsafePtr {
    ptr: u64,
}

// MOVE
#[derive(uniffi::Record)]
pub struct ShadowSocksExposed {
    address: Vec<u8>,
    port: u16,
    password: String,
    cipher: String,
}
impl ShadowSocksExposed {
    pub fn socket(self) -> Shadowsocks {
        // SAFETY: `addr` pointer must be non-null, aligned, and point to at least addr_len bytes
        let endpoint = if let Some(ip_address) =
            unsafe { parse_ip_addr(self.address.as_ptr(), self.address.len()) }
        {
            SocketAddr::new(ip_address, self.port)
        } else {
            todo!();
        };
        let cipher = ShadowsocksCipher::new(&self.cipher).unwrap();

        Shadowsocks::new(endpoint, cipher, self.password)
    }
}

#[uniffi::export(with_foreign)]
pub trait BridgeProvider: Sync + Send {
    fn get_bridges(&self) -> Option<ShadowSocksExposed>;
}

#[uniffi::export(with_foreign)]
pub trait ApiContextCallback: Send + Sync {
    fn access_method_change(&self, context: Arc<dyn ApiContextCallbackContext>, uuid: Vec<u8>);
}

#[uniffi::export(with_foreign)]
pub trait ApiContextCallbackContext: Send + Sync {}

#[derive(uniffi::Object)]
pub struct ApiContext {
    api_client: Runtime<IOSAddressCacheBacking>,
    rest_client: MullvadRestHandle,
    access_mode_handler: AccessModeSelectorHandle,
}
#[uniffi::export]
impl ApiContext {
    #[uniffi::constructor]
    pub fn new_tls_disabled(
        host: String,
        address: String,
        domain: String,
        bridge_provider: Arc<dyn BridgeProvider>,
        settings_provider: Arc<SwiftAccessMethodSettingsContext>,
        access_method_change_callback: Option<Arc<dyn ApiContextCallback>>,
        access_method_change_context: Arc<dyn ApiContextCallbackContext>,
    ) -> Self {
        Self::new_inner(
            host,
            address,
            domain,
            #[cfg(feature = "api-override")]
            true,
            bridge_provider,
            settings_provider,
            access_method_change_callback,
            access_method_change_context,
        )
    }
}

#[uniffi::export]
impl ApiContext {
    // TODO: this needs to be removed
    pub fn unsafe_raw(self: Arc<ApiContext>) -> SwiftApiContext {
        let clone = self.clone();
        SwiftApiContext {
            ptr: Arc::into_raw(clone) as u64,
        }
    }
    #[uniffi::constructor]
    pub fn new(
        host: String,
        address: String,
        domain: String,
        bridge_provider: Arc<dyn BridgeProvider>,
        settings_provider: Arc<SwiftAccessMethodSettingsContext>,
        access_method_change_callback: Option<Arc<dyn ApiContextCallback>>,
        access_method_change_context: Arc<dyn ApiContextCallbackContext>,
    ) -> Self {
        #[cfg(feature = "api-override")]
        return Self::new_inner(
            host,
            address,
            domain,
            false,
            bridge_provider,
            settings_provider,
            access_method_change_callback,
            access_method_change_context,
        );
        #[cfg(not(feature = "api-override"))]
        Self::new_inner(
            host,
            address,
            domain,
            bridge_provider,
            settings_provider,
            access_method_change_callback,
            access_method_change_context,
        )
    }
}
impl ApiContext {
    fn new_inner(
        host: String,
        address: String,
        domain: String,
        #[cfg(feature = "api-override")] disable_tls: bool,
        bridge_provider: Arc<dyn BridgeProvider>,
        settings_provider: Arc<SwiftAccessMethodSettingsContext>,
        access_method_change_callback: Option<Arc<dyn ApiContextCallback>>,
        access_method_change_context: Arc<dyn ApiContextCallbackContext>,
    ) -> Self {
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

        let access_method_settings = settings_provider.convert_access_method().unwrap();
        let encrypted_dns_proxy_state = EncryptedDnsProxyState::default();

        tokio_handle.clone().block_on(async move {
            let (tx, mut rx) = mpsc::unbounded::<(AccessMethodEvent, oneshot::Sender<()>)>();

            // SAFETY: The callback is expected to be called from the Swift side
            if let Some(callback) = access_method_change_callback {
                tokio::spawn(async move {
                    let access_method_change_context = access_method_change_context;
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
                        // TODO: see if we can remove allocation
                        let uuid_bytes = uuid.as_bytes().to_vec();
                        // SAFETY: The callback is expected to be safe to call
                        callback
                            .access_method_change(access_method_change_context.clone(), uuid_bytes);
                    }
                });
            }

            // It is imperative that the REST runtime is created within an async context, otherwise
            // ApiAvailability panics.

            let api_client = mullvad_api::Runtime::with_cache_backing(
                tokio_handle,
                &endpoint,
                Arc::new(IOSAddressCacheBacking {}),
            )
            .await;
            let method_resolver: SwiftAccessMethodResolver = SwiftAccessMethodResolver::new(
                endpoint.clone(),
                domain,
                encrypted_dns_proxy_state,
                bridge_provider,
                api_client.address_cache().clone(),
            );

            let (access_mode_handler, access_mode_provider) = AccessModeSelector::spawn(
                method_resolver,
                access_method_settings,
                #[cfg(feature = "api-override")]
                endpoint.clone(),
                tx,
            )
            .await
            .expect("Could now spawn AccessModeSelector");
            let rest_client = api_client.mullvad_rest_handle(access_mode_provider);

            ApiContext {
                api_client,
                rest_client,
                access_mode_handler,
            }
        })
    }
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

    pub fn address_cache(&self) -> &AddressCache<IOSAddressCacheBacking> {
        self.api_client.address_cache()
    }
}

/// An opaque pointer that exists only to be passed from the caller to a callback through the ABI
struct ForeignPtr {
    ptr: *const c_void,
}
/// allow this to be passed across thread boundaries
/// SAFETY: the user of `ForeignPtr` must ensure that it is safe to use the pointer from different
/// threads.
unsafe impl Send for ForeignPtr {}

/// Called by Swift to set the available access methods
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_update_access_methods(
    api_context: SwiftApiContext,
    settings_wrapper: SwiftAccessMethodSettingsWrapper,
) {
    // SAFETY: `settings_wrapper` must be a valid instance of SwiftAccessMethodSettingsWrapper
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

/// Called by Swift to trigger a fetching and caching of addresses
///
/// # SAFETY
///
/// this takes no arguments other than the API context. The API context
/// needs to be valid, and the function should not be called concurrently.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_update_address_cache(swift_api_context: SwiftApiContext) {
    let api_context = swift_api_context.rust_context();
    let cloned_context = api_context.clone();
    let handle = cloned_context.api_client.handle();
    handle.spawn(async move {
        let api_proxy = ApiProxy::new(api_context.rest_handle());

        match api_proxy.get_api_addrs().await {
            Ok(new_addrs) => {
                if let Some(addr) = new_addrs.first() {
                    log::debug!("Fetched new API address {:?}", addr,);
                    if let Err(err) = api_context.address_cache().set_address(*addr).await {
                        log::error!("Failed to save newly updated API address: {}", err);
                    }
                } else {
                    log::error!("API returned no API addresses");
                }
            }
            Err(err) => {
                log::error!("Failed to fetch new API addresses: {}", err,);
            }
        }
    });
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
