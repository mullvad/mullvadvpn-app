use std::os::raw::c_char;
use std::{ffi::CStr, sync::Arc};

use mullvad_api::{
    AccountsProxy,
    rest::{self, MullvadRestHandle},
};

use crate::api_client::ApiContext;

use super::{
    SwiftApiContext,
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    do_request, do_request_with_empty_body,
    response::SwiftMullvadApiResponse,
    retry_strategy::{LegacySwiftRetryStrategy, RetryStrategy},
};

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `account_number` must be a pointer to a null terminated string.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[uniffi::export]
pub fn mullvad_ios_get_account(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    account_number: String,
) -> SwiftCancelHandle {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion| match mullvad_ios_get_account_inner(
            api_context.rest_handle(),
            retry_strategy,
            account_number,
        )
        .await
        {
            Ok(response) => completion.finish(response),
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(SwiftMullvadApiResponse::rest_error(err));
            }
        },
    )
    .into_swift()
}

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[uniffi::export]
pub fn mullvad_ios_create_account(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
) -> SwiftCancelHandle {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_create_account_inner(api_context.rest_handle(), retry_strategy).await
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

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `account_number` must be a pointer to a null terminated string.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[uniffi::export]
pub fn mullvad_ios_delete_account(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    account_number: String,
) -> SwiftCancelHandle {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_delete_account_inner(
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

async fn mullvad_ios_get_account_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = AccountsProxy::new(rest_client);
    let future_factory = || api.get_data_response(account_number.clone());

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_create_account_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = AccountsProxy::new(rest_client);
    let future_factory = || api.create_account_response();

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_delete_account_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = AccountsProxy::new(rest_client);
    let future_factory = || api.delete_account(account_number.clone());

    do_request_with_empty_body(retry_strategy, future_factory).await
}
