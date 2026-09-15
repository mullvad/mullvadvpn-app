use crate::api_client::access_method_settings::{
    ShadowsocksBridgeProvider, SwiftAccessMethodSettingsContext,
};
use access_method_resolver::{IOSAddressCacheBacking, SwiftAccessMethodResolver};
use futures::{
    StreamExt,
    channel::{mpsc, oneshot},
};
use mullvad_api::{
    ApiEndpoint, ApiProxy, Runtime,
    access_mode::{AccessMethodEvent, AccessModeSelector, AccessModeSelectorHandle},
    rest::{self, MullvadRestHandle},
};
use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState;
use mullvad_types::access_method::Id;
use response::ApiResponse;
use retry_strategy::RetryStrategy;
use std::{future::Future, sync::Arc};
use talpid_future::retry::retry_future;

mod access_method_resolver;
mod access_method_settings;
mod account;
mod api;
mod cancellation;
mod device;
pub(super) mod helpers;
mod mock;
mod problem_report;
mod response;
mod retry_strategy;
mod storekit;
mod swift_data;

#[uniffi::export(with_foreign)]
pub trait AccessMethodChangeCallback: Send + Sync {
    fn access_method_changed_to(&self, uuid: Vec<u8>);
}

#[derive(uniffi::Object)]
pub struct ApiContext {
    api_client: Runtime<IOSAddressCacheBacking>,
    rest_client: MullvadRestHandle,
    access_mode_handler: AccessModeSelectorHandle,
    access_method_change_listeners: Vec<Arc<dyn AccessMethodChangeCallback>>,
}
#[uniffi::export]
impl ApiContext {
    #[uniffi::constructor]
    pub fn new(
        host: String,
        address: String,
        domain: String,
        disable_tls: bool,
        bridge_provider: Arc<dyn ShadowsocksBridgeProvider>,
        settings_provider: Arc<SwiftAccessMethodSettingsContext>,
        access_method_change_listeners: Vec<Arc<dyn AccessMethodChangeCallback>>,
    ) -> Arc<Self> {
        _ = disable_tls;
        Self::new_inner(
            host,
            address,
            domain,
            #[cfg(feature = "api-override")]
            disable_tls,
            bridge_provider,
            settings_provider,
            access_method_change_listeners,
        )
    }
}
impl ApiContext {
    fn new_inner(
        host: String,
        address: String,
        domain: String,
        #[cfg(feature = "api-override")] disable_tls: bool,
        bridge_provider: Arc<dyn ShadowsocksBridgeProvider>,
        settings_provider: Arc<SwiftAccessMethodSettingsContext>,
        access_method_change_listeners: Vec<Arc<dyn AccessMethodChangeCallback>>,
    ) -> Arc<Self> {
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

            let context = Arc::new(ApiContext {
                api_client,
                rest_client,
                access_mode_handler,
                access_method_change_listeners,
            });

            tokio::spawn({
                let context = context.clone();
                async move {
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
                        let uuid_bytes = uuid.as_bytes().to_vec();
                        for callback in &context.access_method_change_listeners {
                            callback.access_method_changed_to(uuid_bytes.clone());
                        }
                    }
                }
            });

            context
        })
    }
}
impl ApiContext {
    pub fn rest_handle(&self) -> MullvadRestHandle {
        self.rest_client.clone()
    }
}
#[uniffi::export]
impl ApiContext {
    /// Replaces the current set of access methods with `access_methods.
    ///
    /// This function will block the current thread until it is complete,
    /// make sure to not call this from a UI Thread if possible.
    pub fn update_access_methods(&self, settings_wrapper: Arc<SwiftAccessMethodSettingsContext>) {
        let access_methods = settings_wrapper.settings.clone();
        _ = self.api_client.handle().block_on(async {
            self.access_mode_handler
                .update_access_methods(access_methods)
                .await
        });
    }

    /// Sets the access method referenced by `id` as currently in use.
    ///
    /// This function will block the current thread until it is complete,
    /// make sure to not call this from a UI Thread if possible.
    pub fn use_access_method(&self, id: String) {
        let Some(id) = Id::from_string(id) else {
            return;
        };
        _ = self
            .api_client
            .handle()
            .block_on(async { self.access_mode_handler.use_access_method(id).await });
    }

    /// Called by Swift to trigger a fetching and caching of addresses
    pub fn update_address_cache(self: Arc<Self>) {
        let cloned_context = self.clone();
        let handle = cloned_context.api_client.handle();
        handle.spawn(async move {
            let api_proxy = ApiProxy::new(self.rest_handle());

            match api_proxy.get_api_addrs().await {
                Ok(new_addrs) => {
                    if let Some(addr) = new_addrs.first() {
                        log::debug!("Fetched new API address {:?}", addr,);
                        if let Err(err) = self.api_client.address_cache().set_address(*addr).await {
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
}

async fn do_request<F, T>(
    retry_strategy: RetryStrategy,
    future_factory: F,
) -> Result<ApiResponse, rest::Error>
where
    F: Fn() -> T,
    T: Future<Output = Result<rest::Response<hyper::body::Incoming>, rest::Error>>,
{
    let response = retry_request(retry_strategy, future_factory).await?;
    ApiResponse::with_body(response).await
}

async fn do_request_with_empty_body<F, T>(
    retry_strategy: RetryStrategy,
    future_factory: F,
) -> Result<ApiResponse, rest::Error>
where
    F: Fn() -> T,
    T: Future<Output = Result<(), rest::Error>>,
{
    retry_request(retry_strategy, future_factory).await?;
    Ok(ApiResponse::ok())
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
