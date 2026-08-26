use chrono::{DateTime, Utc};
use std::ffi::c_void;

use mullvad_api::relay_list_transparency::{RelayListDigest, SigsumPayload};
use mullvad_api::{
    ApiProxy, RelayListProxy,
    rest::{self, MullvadRestHandle},
};
use mullvad_types::access_method::AccessMethodSetting;

use super::{
    SwiftApiContext,
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
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
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_get_addresses(
    api_context: SwiftApiContext,
    retry_strategy: SwiftRetryStrategy,
) -> SwiftCancelHandle {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_get_addresses_inner(api_context.rest_handle(), retry_strategy).await {
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
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_api_addrs_available(
    api_context: SwiftApiContext,
    retry_strategy: SwiftRetryStrategy,
    access_method_setting: *const c_void,
) -> SwiftCancelHandle {
    // SAFETY: `access_method_setting` must be a raw pointer resulting from a call to `convert_builtin_access_method_setting`
    let access_method_setting: AccessMethodSetting =
        unsafe { *Box::from_raw(access_method_setting as *mut _) };

    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| match api_context
            .access_mode_handler
            .resolve(access_method_setting.clone())
            .await
        {
            Ok(Some(resolved_connection_mode)) => {
                let oneshot_client = api_context
                    .api_client
                    .mullvad_rest_handle(resolved_connection_mode.connection_mode.into_provider());

                match mullvad_ios_api_addrs_available_inner(oneshot_client, retry_strategy).await {
                    Ok(_) => completion_handler.finish(SwiftMullvadApiResponse::ok()),
                    Err(err) => {
                        log::error!("{err:?}");
                        completion_handler.finish(SwiftMullvadApiResponse::rest_error(err));
                    }
                }
            }
            Ok(None) => {
                log::error!("Invalid access method configuration, {access_method_setting:?}");
                completion_handler.finish(SwiftMullvadApiResponse::access_method_error(
                    mullvad_api::access_mode::Error::Resolve {
                        access_method: access_method_setting.access_method,
                    },
                ));
            }
            Err(err) => {
                log::error!("{err:?}");
                completion_handler.finish(SwiftMullvadApiResponse::access_method_error(err));
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
/// `digest` must point to valid checksum consisting of 32 bytes, or null if not used.
///
/// `digest_timestamp` should be a valid UTC timestamp, or 0 if not used.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_get_relays(
    api_context: SwiftApiContext,
    retry_strategy: SwiftRetryStrategy,
    digest: *const u8,
    digest_timestamp: i64,
) -> SwiftCancelHandle {
    let digest = if !digest.is_null() {
        let mut bytes = [0; 32];
        // Safety: See `digest` above
        let source = unsafe { std::slice::from_raw_parts(digest, 32) };
        bytes.as_mut_slice().copy_from_slice(source);
        Some(RelayListDigest::new(bytes))
    } else {
        None
    };
    let timestamp = if digest_timestamp != 0 {
        DateTime::from_timestamp_millis(digest_timestamp)
    } else {
        None
    };
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_get_relays_inner(
                api_context.rest_handle(),
                retry_strategy,
                digest,
                timestamp,
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
    digest: Option<RelayListDigest>,
    digest_timestamp: Option<DateTime<Utc>>,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = RelayListProxy::new(rest_client);
    let sigsum_payload = digest
        .zip(digest_timestamp)
        .map(|(digest, timestamp)| SigsumPayload::new(digest, timestamp));

    let future_factory = || api.relay_list_response(sigsum_payload.clone());

    let response = retry_request(retry_strategy, future_factory).await?;
    SwiftMullvadApiResponse::with_sigsum_verified_body(response)
}

async fn mullvad_ios_api_addrs_available_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<bool, rest::Error> {
    let api = ApiProxy::new(rest_client);

    let future_factory = || api.api_addrs_available();
    retry_request(retry_strategy, future_factory).await
}
