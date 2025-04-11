use std::os::raw::c_char;

use mullvad_api::{
    rest::{self, MullvadRestHandle},
    AccountsProxy,
};
use mullvad_types::account::AccountNumber;
use talpid_future::retry::retry_future;

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    completion::{CompletionCookie, SwiftCompletionHandler},
    do_request,
    response::SwiftMullvadApiResponse,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
    SwiftApiContext,
};

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_ios_init_new`.
///
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is
/// safe because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this
/// type is intended to be used.
///
/// `account` must be a pointer to a null terminated string to the account number
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_ios_init_storekit_payment(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account: *const c_char,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie::new(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let completion = completion_handler.clone();

    let account = unsafe { std::ffi::CStr::from_ptr(account.cast()) };
    let Ok(account) = account.to_str() else {
        completion_handler.finish(SwiftMullvadApiResponse::invalid_input(
            c"Invalid account string",
        ));
        return SwiftCancelHandle::empty();
    };
    let account = AccountNumber::from(account);

    let task = tokio_handle.clone().spawn(async move {
        match mullvad_ios_init_storekit_payment_inner(
            api_context.rest_handle(),
            retry_strategy,
            account,
        )
        .await
        {
            Ok(response) => completion.finish(response),
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(SwiftMullvadApiResponse::rest_error(err));
            }
        }
    });

    RequestCancelHandle::new(task, completion_handler.clone()).into_swift()
}

async fn mullvad_ios_init_storekit_payment_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account: AccountNumber,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let account_proxy = AccountsProxy::new(rest_client);

    let future_factory = || account_proxy.init_storekit_payment(account.clone());

    do_request(retry_strategy, future_factory).await
}

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_ios_init_new`.
///
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is
/// safe because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this
/// type is intended to be used.
///
/// `account` must be a pointer to a null terminated string to the account number
///
/// `body` must be a pointer to the body content
///
/// `body_size` must be the size of the body
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_ios_check_storekit_payment(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account: *const c_char,
    body: *const u8,
    body_size: usize,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie::new(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let completion = completion_handler.clone();

    let account = unsafe { std::ffi::CStr::from_ptr(account.cast()) };
    let Ok(account) = account.to_str() else {
        completion_handler.finish(SwiftMullvadApiResponse::invalid_input(
            c"Invalid account string",
        ));
        return SwiftCancelHandle::empty();
    };
    let account = AccountNumber::from(account);

    let body = unsafe { std::slice::from_raw_parts(body, body_size) }.to_vec();
    let task = tokio_handle.clone().spawn(async move {
        match mullvad_ios_check_storekit_payment_inner(
            api_context.rest_handle(),
            retry_strategy,
            account,
            body,
        )
        .await
        {
            Ok(response) => completion.finish(response),
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(SwiftMullvadApiResponse::rest_error(err));
            }
        }
    });

    RequestCancelHandle::new(task, completion_handler.clone()).into_swift()
}

async fn mullvad_ios_check_storekit_payment_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account: AccountNumber,
    body: Vec<u8>,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let account_proxy = AccountsProxy::new(rest_client);

    let future_factory = || account_proxy.check_storekit_payment(account.clone(), body.clone());

    do_request(retry_strategy, future_factory).await
}
