use libc::c_char;
use mullvad_api::{
    rest::{self, MullvadRestHandle},
    DevicesProxy,
};

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    completion::{CompletionCookie, SwiftCompletionHandler},
    do_request, do_request_with_empty_body, get_string,
    response::SwiftMullvadApiResponse,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
    SwiftApiContext,
};
use std::ptr;
use talpid_types::net::wireguard;
use talpid_types::net::wireguard::PublicKey;

/// Get device info via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_ios_init_new`.
///
/// This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
/// object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_ios_completion_finish`
/// when completion finishes (in completion.finish).
///
/// the `account_number` must be a pointer to a null terminated string.
/// the `identifier` must be a pointer to a null terminated string.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_get_device(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account_number: *const c_char,
    identifier: *const c_char,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie::new(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };
    let account_number = get_string(account_number);
    let identifier = get_string(identifier);

    let completion = completion_handler.clone();
    let task = tokio_handle.spawn(async move {
        match mullvad_ios_get_device_inner(
            api_context.rest_handle(),
            retry_strategy,
            account_number,
            identifier,
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

    RequestCancelHandle::new(task, completion_handler).into_swift()
}

/// Get devices info via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
/// object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
/// when completion finishes (in completion.finish).
///
/// the `account_number` must be a pointer to a null terminated string.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_get_devices(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account_number: *const c_char,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie::new(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    // Safety: The caller must guarantee that `retry_strategy` is not null and has not been freed
    let retry_strategy = unsafe { retry_strategy.into_rust() };
    let account_number = get_string(account_number);

    let completion = completion_handler.clone();
    let task = tokio_handle.spawn(async move {
        match mullvad_ios_get_devices_inner(
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

    RequestCancelHandle::new(task, completion_handler).into_swift()
}

/// create device via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
/// object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
/// when completion finishes (in completion.finish).
///
/// the `account_number` must be a pointer to a null terminated string.
/// the `identifier` must be a pointer to a null terminated string.
/// the `public_key` pointer must be a valid pointer to 32 unsigned bytes.
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_create_device(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account_number: *const c_char,
    public_key: *const u8,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie::new(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    // Safety: The caller must guarantee that `retry_strategy` is not null and has not been freed
    let retry_strategy = unsafe { retry_strategy.into_rust() };
    let account_number = get_string(account_number);
    // Safety: `public_key` pointer must be a valid pointer to 32 unsigned bytes.
    let pub_key: [u8; 32] = unsafe { ptr::read(public_key as *const [u8; 32]) };

    let completion = completion_handler.clone();
    let task = tokio_handle.spawn(async move {
        match mullvad_ios_create_device_inner(
            api_context.rest_handle(),
            retry_strategy,
            account_number,
            PublicKey::from(pub_key),
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

    RequestCancelHandle::new(task, completion_handler).into_swift()
}

/// delete device via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
/// object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
/// when completion finishes (in completion.finish).
///
/// the `account_number` must be a pointer to a null terminated string.
/// the `identifier` must be a pointer to a null terminated string.
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_delete_device(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account_number: *const c_char,
    identifier: *const c_char,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie::new(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    // Safety: The caller must guarantee that `retry_strategy` is not null and has not been freed
    let retry_strategy = unsafe { retry_strategy.into_rust() };
    let account_number = get_string(account_number);
    let identifier = get_string(identifier);

    let completion = completion_handler.clone();
    let task = tokio_handle.spawn(async move {
        match mullvad_ios_delete_device_inner(
            api_context.rest_handle(),
            retry_strategy,
            account_number,
            identifier,
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

    RequestCancelHandle::new(task, completion_handler).into_swift()
}

/// rotate device key via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
/// object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
/// when completion finishes (in completion.finish).
///
/// the `account_number` must be a pointer to a null terminated string.
/// the `identifier` must be a pointer to a null terminated string.
/// the `public_key` pointer must be a valid pointer to 32 unsigned bytes.
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_rotate_device_key(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account_number: *const c_char,
    identifier: *const c_char,
    public_key: *const u8,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie::new(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    // Safety: The caller must guarantee that `retry_strategy` is not null and has not been freed
    let retry_strategy = unsafe { retry_strategy.into_rust() };
    let account_number = get_string(account_number);
    let identifier = get_string(identifier);
    // Safety: `public_key` pointer must be a valid pointer to 32 unsigned bytes.
    let pub_key: [u8; 32] = unsafe { ptr::read(public_key as *const [u8; 32]) };

    let completion = completion_handler.clone();
    let task = tokio_handle.spawn(async move {
        match mullvad_ios_rotate_device_key_inner(
            api_context.rest_handle(),
            retry_strategy,
            account_number,
            identifier,
            PublicKey::from(pub_key),
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

    RequestCancelHandle::new(task, completion_handler).into_swift()
}

async fn mullvad_ios_get_device_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    identifier: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.get_device(account_number.clone(), identifier.clone());

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_get_devices_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.get_devices(account_number.clone());

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

    let future_factory = || {
        api.replace_wg_key_reponse(
            account_number.clone(),
            identifier.clone(),
            pub_key.to_owned(),
        )
    };

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_create_device_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
    pub_key: wireguard::PublicKey,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = DevicesProxy::new(rest_client);

    let future_factory = || api.create_reponse(account_number.clone(), pub_key.to_owned());

    do_request(retry_strategy, future_factory).await
}
