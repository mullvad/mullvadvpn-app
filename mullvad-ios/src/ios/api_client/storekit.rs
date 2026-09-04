use std::os::raw::c_char;

use mullvad_api::{
    AccountsProxy,
    rest::{self, MullvadRestHandle},
};
use mullvad_types::account::AccountNumber;

use super::{
    SwiftApiContext,
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    do_request, get_string,
    response::SwiftMullvadApiResponse,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_init_storekit_payment(
    api_context: SwiftApiContext,
    retry_strategy: SwiftRetryStrategy,
    account_number: *const c_char,
) -> SwiftCancelHandle {
    // SAFETY: See param documentation for `account_number`.
    let account_number = AccountNumber::from(unsafe { get_string(account_number) });

    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_init_storekit_payment_inner(
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

async fn mullvad_ios_init_storekit_payment_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: AccountNumber,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let account_proxy = AccountsProxy::new(rest_client);

    let future_factory = || account_proxy.init_storekit_payment(account_number.clone());

    do_request(retry_strategy, future_factory).await
}

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// `body` must be a pointer to a contiguous memory segment
///
/// `body_size` must be the size of the body
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_check_storekit_payment(
    api_context: SwiftApiContext,
    retry_strategy: SwiftRetryStrategy,
    body: *const u8,
    body_size: usize,
) -> SwiftCancelHandle {
    // SAFETY: See param documentation for `body`.
    let body = unsafe { std::slice::from_raw_parts(body, body_size) }.to_vec();
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_check_storekit_payment_inner(
                api_context.rest_handle(),
                retry_strategy,
                body,
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

async fn mullvad_ios_check_storekit_payment_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    body: Vec<u8>,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let account_proxy = AccountsProxy::new(rest_client);

    let future_factory = || account_proxy.check_storekit_payment(body.clone());

    do_request(retry_strategy, future_factory).await
}
