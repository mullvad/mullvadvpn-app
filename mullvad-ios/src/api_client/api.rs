use std::ffi::{CStr, c_void};
use std::os::raw::c_char;

use mullvad_api::{
    ApiProxy, ETag, RelayListProxy,
    rest::{self, MullvadRestHandle},
};
use mullvad_types::access_method::AccessMethodSetting;

use super::{
    SwiftApiContext,
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    completion::{CompletionCookie, SwiftCompletionHandler},
    do_request,
    response::SwiftMullvadApiResponse,
    retry_request,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
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
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_get_addresses(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
) -> SwiftCancelHandle {
    let completion_handler =
        SwiftCompletionHandler::new(unsafe { CompletionCookie::new(completion_cookie) });

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
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_api_addrs_available(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    access_method_setting: *const c_void,
) -> SwiftCancelHandle {
    let completion_handler =
        SwiftCompletionHandler::new(unsafe { CompletionCookie::new(completion_cookie) });

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.rust_context();
    // SAFETY: See notes for `into_rust`
    let retry_strategy = unsafe { retry_strategy.into_rust() };
    let completion = completion_handler.clone();
    // SAFETY: `access_method_setting` must be a raw pointer resulting from a call to `convert_builtin_access_method_setting`
    let access_method_setting: AccessMethodSetting =
        unsafe { *Box::from_raw(access_method_setting as *mut _) };

    let task = tokio_handle.clone().spawn(async move {
        match api_context
            .access_mode_handler
            .resolve(access_method_setting.clone())
            .await
        {
            Ok(Some(resolved_connection_mode)) => {
                let oneshot_client = api_context
                    .api_client
                    .mullvad_rest_handle(resolved_connection_mode.connection_mode.into_provider());

                match mullvad_ios_api_addrs_available_inner(oneshot_client, retry_strategy).await {
                    Ok(_) => completion.finish(SwiftMullvadApiResponse::ok()),
                    Err(err) => {
                        log::error!("{err:?}");
                        completion.finish(SwiftMullvadApiResponse::rest_error(err));
                    }
                }
            }
            Ok(None) => {
                log::error!("Invalid access method configuration, {access_method_setting:?}");
                completion.finish(SwiftMullvadApiResponse::access_method_error(
                    mullvad_api::access_mode::Error::Resolve {
                        access_method: access_method_setting.access_method,
                    },
                ));
            }
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(SwiftMullvadApiResponse::access_method_error(err));
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
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_get_relays(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    etag: *const c_char,
) -> SwiftCancelHandle {
    let completion_handler =
        SwiftCompletionHandler::new(unsafe { CompletionCookie::new(completion_cookie) });

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.rust_context();
    // SAFETY: See notes for `into_rust`
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let mut maybe_etag: Option<ETag> = None;
    if !etag.is_null() {
        // SAFETY: See param documentation for `etag`.
        let unwrapped_tag = unsafe { CStr::from_ptr(etag.cast()) }.to_str().unwrap();
        maybe_etag = Some(ETag(String::from(unwrapped_tag)));
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
    etag: Option<ETag>,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = RelayListProxy::new(rest_client);

    let future_factory = || api.relay_list_response(etag.clone());

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_api_addrs_available_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<bool, rest::Error> {
    let api = ApiProxy::new(rest_client);

    let future_factory = || api.api_addrs_available();
    retry_request(retry_strategy, future_factory).await
}
