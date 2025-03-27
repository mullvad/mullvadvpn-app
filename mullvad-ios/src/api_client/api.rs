use std::ffi::CStr;
use std::os::raw::c_char;

use mullvad_api::{
    rest::{self, MullvadRestHandle},
    ApiProxy, RelayListProxy,
};

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
/// by calling `mullvad_api_init_new`.
///
/// This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
/// object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
/// when completion finishes (in completion.finish).
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_ios_get_addresses(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie::new(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.rust_context();
    // SAFETY: See notes for `into_rust`
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let completion = completion_handler.clone();
    let task = tokio_handle.clone().spawn(async move {
        match mullvad_ios_get_addresses_inner(api_context.rest_handle(), retry_strategy).await {
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
/// This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
/// object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
/// when completion finishes (in completion.finish).
///
/// `etag` must be a pointer to a null terminated string.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_ios_get_relays(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    etag: *const c_char,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie::new(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.rust_context();
    // SAFETY: See notes for `into_rust`
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let mut maybe_etag: Option<String> = None;
    if !etag.is_null() {
        // SAFETY: See param documentation for `etag`.
        let unwrapped_tag = unsafe { CStr::from_ptr(etag.cast()) }.to_str().unwrap();
        maybe_etag = Some(String::from(unwrapped_tag));
    }

    let completion = completion_handler.clone();
    let task = tokio_handle.clone().spawn(async move {
        match mullvad_ios_get_relays_inner(api_context.rest_handle(), retry_strategy, maybe_etag)
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

async fn mullvad_ios_get_addresses_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = ApiProxy::new(rest_client);

    let future_factory = || api.get_api_addrs_response();

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_get_relays_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    etag: Option<String>,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = RelayListProxy::new(rest_client);

    let future_factory = || api.relay_list_response(etag.clone());

    do_request(retry_strategy, future_factory).await
}
