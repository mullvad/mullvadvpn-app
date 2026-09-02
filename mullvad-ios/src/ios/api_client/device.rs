use mullvad_api::{
    DevicesProxy,
    rest::{self, MullvadRestHandle},
};

use crate::api_client::{ApiContext, retry_strategy::mullvad_api_retry_strategy_never};

use super::{
    cancellation::RequestCancelHandle, do_request, do_request_with_empty_body,
    response::ApiResponse, retry_strategy::RetryStrategy,
};
use std::sync::Arc;
use talpid_types::net::wireguard;
use talpid_types::net::wireguard::PublicKey;

#[uniffi::export]
impl ApiContext {
    /// Get device info via the Mullvad API client.
    pub fn get_device(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
        identifier: String,
    ) -> Arc<RequestCancelHandle> {
        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| match get_device_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
                identifier,
            )
            .await
            {
                Ok(response) => completion_handler.finish(Arc::new(response)),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(Arc::new(ApiResponse::rest_error(err)));
                }
            },
        )
    }

    /// Get devices info via the Mullvad API client.
    pub fn get_devices(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
    ) -> Arc<RequestCancelHandle> {
        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| match get_devices_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
            )
            .await
            {
                Ok(response) => completion_handler.finish(Arc::new(response)),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(Arc::new(ApiResponse::rest_error(err)));
                }
            },
        )
    }

    /// create device via the Mullvad API client.
    pub fn create_device(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
        public_key: &[u8],
    ) -> Arc<RequestCancelHandle> {
        let Ok(pub_key): Result<[u8; 32], _> = public_key.try_into() else {
            return RequestCancelHandle::new(
                self,
                Arc::new(mullvad_api_retry_strategy_never()),
                async |_, _, completion_handler| {
                    completion_handler.finish(Arc::new(ApiResponse::other("bad public key size")));
                },
            );
        };

        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| match create_device_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
                PublicKey::from(pub_key),
            )
            .await
            {
                Ok(response) => completion_handler.finish(Arc::new(response)),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(Arc::new(ApiResponse::rest_error(err)));
                }
            },
        )
    }

    /// Delete device via the Mullvad API client.
    pub fn delete_device(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
        identifier: String,
    ) -> Arc<RequestCancelHandle> {
        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| match delete_device_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
                identifier,
            )
            .await
            {
                Ok(response) => completion_handler.finish(Arc::new(response)),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(Arc::new(ApiResponse::rest_error(err)));
                }
            },
        )
    }

    /// Rotate device key via the Mullvad API client.
    pub fn rotate_device_key(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
        identifier: String,
        public_key: &[u8],
    ) -> Arc<RequestCancelHandle> {
        let Ok(pub_key): Result<[u8; 32], _> = public_key.try_into() else {
            return RequestCancelHandle::new(
                self,
                Arc::new(mullvad_api_retry_strategy_never()),
                async |_, _, completion_handler| {
                    completion_handler.finish(Arc::new(ApiResponse::other("bad public key size")));
                },
            );
        };

        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| {
                match rotate_device_key_inner(
                    api_context.rest_handle(),
                    retry_strategy,
                    account_number,
                    identifier,
                    PublicKey::from(pub_key),
                )
                .await
                {
                    Ok(response) => completion_handler.finish(Arc::new(response)),
                    Err(err) => {
                        log::error!("{err:?}");
                        completion_handler.finish(Arc::new(ApiResponse::rest_error(err)));
                    }
                }
            },
        )
    }
}

async fn get_device_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    identifier: String,
) -> Result<ApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.get_response(account_number.clone(), identifier.clone());

    do_request(retry_strategy, future_factory).await
}

async fn get_devices_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
) -> Result<ApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.list_response(account_number.clone());

    do_request(retry_strategy, future_factory).await
}

async fn delete_device_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    identifier: String,
) -> Result<ApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.remove(account_number.clone(), identifier.clone());

    do_request_with_empty_body(retry_strategy, future_factory).await
}

async fn rotate_device_key_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    identifier: String,
    pub_key: wireguard::PublicKey,
) -> Result<ApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory =
        || api.replace_wg_key_response(account_number.clone(), identifier.clone(), pub_key.clone());

    do_request(retry_strategy, future_factory).await
}

async fn create_device_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    pub_key: wireguard::PublicKey,
) -> Result<ApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.create_response(account_number.clone(), pub_key.clone());

    do_request(retry_strategy, future_factory).await
}
