use mullvad_api::{
    ProblemReportProxy,
    rest::{self, MullvadRestHandle},
};
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use crate::api_client::ApiContext;

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    do_request_with_empty_body,
    response::SwiftMullvadApiResponse,
    retry_strategy::RetryStrategy,
};

use std::collections::BTreeMap;

/// Send a problem report via the Mullvad API client.
///
/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `retry_strategy` must have been created by a call to either of the following functions
/// `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
///
/// the string properties of `SwiftProblemReportRequest` must be pointers to a null terminated strings.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[uniffi::export]
pub fn mullvad_ios_send_problem_report(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    request: ProblemReportRequest,
) -> SwiftCancelHandle {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_send_problem_report_inner(
                api_context.rest_handle(),
                retry_strategy,
                request,
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

async fn mullvad_ios_send_problem_report_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    problem_report_request: ProblemReportRequest,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = ProblemReportProxy::new(rest_client);

    let future_factory = || {
        api.problem_report(
            &problem_report_request.address,
            &problem_report_request.message,
            &(String::from_utf8_lossy(&problem_report_request.log)),
            &problem_report_request.metadata,
        )
    };

    do_request_with_empty_body(retry_strategy, future_factory).await
}

// TODO THIS SHOULD NOT BE A RECORD!!!! COSTLY
#[derive(uniffi::Record)]
pub struct ProblemReportRequest {
    address: String,
    message: String,
    log: Vec<u8>,
    // TODO can we switch this to a hashmap without breakage?
    metadata: HashMap<String, String>,
}
#[uniffi::export]
pub fn problem_report_request_init(
    address: String,
    message: String,
    log: Vec<u8>,
    metadata: HashMap<String, String>,
) -> ProblemReportRequest {
    ProblemReportRequest {
        address,
        message,
        log,
        metadata,
    }
}

#[derive(uniffi::Object)]
pub struct ProblemReportMetadata {
    inner: Mutex<BTreeMap<String, String>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn swift_problem_report_metadata_new() -> ProblemReportMetadata {
    ProblemReportMetadata {
        inner: Mutex::new(BTreeMap::new()),
    }
}

/// Add key and value pair to the `ProblemReportMetadata`
///
/// # Safety
///
/// `map.inner` must be non-null and point to a valid
/// - `key` must be a null-terminated UTF-8 string, containing LF-separated machines.
/// - `value` must be a valid pointer to some valid and aligned pointer-sized memory.
#[uniffi::export]
pub fn swift_problem_report_metadata_add(
    map: &ProblemReportMetadata,
    key: String,
    value: String,
) -> bool {
    map.inner.lock().unwrap().insert(key, value);
    true
}
