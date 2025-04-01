use std::ffi::CStr;

use mullvad_api::{
    rest::{self, MullvadRestHandle},
    AccountsProxy,
};

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    completion::{CompletionCookie, SwiftCompletionHandler},
    do_request, do_request_with_empty_body,
    response::SwiftMullvadApiResponse,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
    SwiftApiContext,
};

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is
/// safe because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this
/// type is intended to be used.
///
/// `account_number` must be a pointer to a null terminated string.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_api_get_account(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account_number: *const u8,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };
    let account_number = unsafe { CStr::from_ptr(account_number.cast()) }
        .to_str()
        .unwrap();
    let account_number = String::from(account_number);

    let completion = completion_handler.clone();
    let task = tokio_handle.clone().spawn(async move {
        match mullvad_api_get_account_inner(
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
        }
    });

    RequestCancelHandle::new(task, completion_handler.clone()).into_swift()
}

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is
/// safe because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this
/// type is intended to be used.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_api_create_account(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let completion = completion_handler.clone();
    let task = tokio_handle.clone().spawn(async move {
        match mullvad_api_create_account_inner(api_context.rest_handle(), retry_strategy).await {
            Ok(response) => completion.finish(response),
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(SwiftMullvadApiResponse::rest_error(err));
            }
        }
    });

    RequestCancelHandle::new(task, completion_handler.clone()).into_swift()
}

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is
/// safe because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this
/// type is intended to be used.
///
/// `account_number` must be a pointer to a null terminated string.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_api_delete_account(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account_number: *const u8,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };
    let account_number = unsafe { CStr::from_ptr(account_number.cast()) }
        .to_str()
        .unwrap();
    let account_number = String::from(account_number);

    let completion = completion_handler.clone();
    let task = tokio_handle.clone().spawn(async move {
        match mullvad_api_delete_account_inner(
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
        }
    });

    RequestCancelHandle::new(task, completion_handler.clone()).into_swift()
}

async fn mullvad_api_get_account_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = AccountsProxy::new(rest_client);
    let future_factory = || api.get_data_response(account_number.clone());

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_api_create_account_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = AccountsProxy::new(rest_client);
    let future_factory = || api.create_account_response();

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_api_delete_account_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = AccountsProxy::new(rest_client);
    let future_factory = || api.delete_account(account_number.clone());

    do_request_with_empty_body(retry_strategy, future_factory).await
}
