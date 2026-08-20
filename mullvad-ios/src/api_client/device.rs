use mullvad_api::{
    DevicesProxy,
    rest::{self, MullvadRestHandle},
};

use crate::api_client::ApiContext;

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    do_request, do_request_with_empty_body,
    response::SwiftMullvadApiResponse,
    retry_strategy::RetryStrategy,
};
use std::{ptr, sync::Arc};
use talpid_types::net::wireguard;
use talpid_types::net::wireguard::PublicKey;

/// Get device info via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_ios_init_new`.
///
/// the `account_number` must be a pointer to a null terminated string.
/// the `identifier` must be a pointer to a null terminated string.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[uniffi::export]
pub fn mullvad_ios_get_device(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    account_number: String,
    identifier: String,
) -> SwiftCancelHandle {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_get_device_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
                identifier,
            )
            .await
            {
                Ok(response) => completion_handler.finish(response),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(SwiftMullvadApiResponse::rest_error(err));
                }
            }
        },
    )
    .into_swift()
}

/// Get devices info via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// the `account_number` must be a pointer to a null terminated string.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[uniffi::export]
pub fn mullvad_ios_get_devices(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    account_number: String,
) -> SwiftCancelHandle {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_get_devices_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
            )
            .await
            {
                Ok(response) => completion_handler.finish(response),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(SwiftMullvadApiResponse::rest_error(err));
                }
            }
        },
    )
    .into_swift()
}

/// create device via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// the `account_number` must be a pointer to a null terminated string.
/// the `identifier` must be a pointer to a null terminated string.
/// the `public_key` pointer must be a valid pointer to 32 unsigned bytes.
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[uniffi::export]
pub fn mullvad_ios_create_device(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    account_number: String,
    public_key: &[u8],
) -> SwiftCancelHandle {
    // Safety: `public_key` pointer must be a valid pointer to 32 unsigned bytes.
    let pub_key: [u8; 32] = unsafe { ptr::read(public_key as *const [u8] as *const [u8; 32]) };

    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_create_device_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
                PublicKey::from(pub_key),
            )
            .await
            {
                Ok(response) => completion_handler.finish(response),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(SwiftMullvadApiResponse::rest_error(err));
                }
            }
        },
    )
    .into_swift()
}

/// delete device via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// the `account_number` must be a pointer to a null terminated string.
/// the `identifier` must be a pointer to a null terminated string.
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[uniffi::export]
pub fn mullvad_ios_delete_device(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    account_number: String,
    identifier: String,
) -> SwiftCancelHandle {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_delete_device_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
                identifier,
            )
            .await
            {
                Ok(response) => completion_handler.finish(response),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(SwiftMullvadApiResponse::rest_error(err));
                }
            }
        },
    )
    .into_swift()
}

/// rotate device key via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// the `account_number` must be a pointer to a null terminated string.
/// the `identifier` must be a pointer to a null terminated string.
/// the `public_key` pointer must be a valid pointer to 32 unsigned bytes.
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[uniffi::export]
pub fn mullvad_ios_rotate_device_key(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    account_number: String,
    identifier: String,
    public_key: &[u8],
) -> SwiftCancelHandle {
    // SAFETY: `public_key` pointer must be a valid pointer to 32 unsigned bytes.
    let pub_key: [u8; 32] = unsafe { ptr::read(public_key as *const [u8] as *const [u8; 32]) };

    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_rotate_device_key_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
                identifier,
                PublicKey::from(pub_key),
            )
            .await
            {
                Ok(response) => completion_handler.finish(response),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(SwiftMullvadApiResponse::rest_error(err));
                }
            }
        },
    )
    .into_swift()
}

async fn mullvad_ios_get_device_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    identifier: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.get_response(account_number.clone(), identifier.clone());

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_get_devices_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.list_response(account_number.clone());

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_delete_device_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    identifier: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.remove(account_number.clone(), identifier.clone());

    do_request_with_empty_body(retry_strategy, future_factory).await
}

async fn mullvad_ios_rotate_device_key_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    identifier: String,
    pub_key: wireguard::PublicKey,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory =
        || api.replace_wg_key_response(account_number.clone(), identifier.clone(), pub_key.clone());

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_create_device_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    pub_key: wireguard::PublicKey,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.create_response(account_number.clone(), pub_key.clone());

    do_request(retry_strategy, future_factory).await
}
