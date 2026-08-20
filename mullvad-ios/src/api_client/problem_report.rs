use mullvad_api::{
    ProblemReportProxy,
    rest::{self, MullvadRestHandle},
};
use std::ffi::CStr;
use std::os::raw::c_char;

use super::{
    SwiftApiContext,
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    do_request_with_empty_body, get_string,
    response::SwiftMullvadApiResponse,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
};

use mullvad_api::rest::Error;
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_send_problem_report(
    api_context: SwiftApiContext,
    retry_strategy: SwiftRetryStrategy,
    request: SwiftProblemReportRequest,
) -> SwiftCancelHandle {
    // SAFETY: See safety notes for `from_swift_parameters`
    let result = unsafe { ProblemReportRequest::from_swift_parameters(request) };

    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            let Some(problem_report_request) = result else {
                let err = Error::ApiError(
                    rest::StatusCode::BAD_REQUEST,
                    "Failed to send problem report: invalid address, message, or log data."
                        .to_string(),
                );
                log::error!("{err:?}");
                completion_handler.finish(SwiftMullvadApiResponse::rest_error(err));
                return;
            };

            match mullvad_ios_send_problem_report_inner(
                api_context.rest_handle(),
                retry_strategy,
                problem_report_request,
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

#[repr(C)]
pub struct SwiftProblemReportRequest {
    address: *const c_char,
    message: *const c_char,
    log: *const c_char,
    metadata: ProblemReportMetadata,
}

struct ProblemReportRequest {
    address: String,
    message: String,
    log: Vec<u8>,
    metadata: BTreeMap<String, String>,
}

impl ProblemReportRequest {
    // SAFETY: the members of `SwiftProblemReportRequest` must point to null-terminated strings
    unsafe fn from_swift_parameters(request: SwiftProblemReportRequest) -> Option<Self> {
        // SAFETY: caller guarantees that `request.address` is a valid C string pointer
        let address = unsafe { get_string(request.address) };
        // SAFETY: caller guarantees that `request.message` is a valid C string pointer
        let message = unsafe { get_string(request.message) };
        // SAFETY: caller guarantees that `request.log` is a valid C string pointer
        let log = unsafe { get_string(request.log) }.into();

        let metadata = if request.metadata.inner.is_null() {
            BTreeMap::new()
        } else {
            let swift_map = &request.metadata;
            let mut converted_map = BTreeMap::new();

            // SAFETY: caller guarantees that `swift_map.inner` is a valid pointer to a [`Map`]
            if let Some(inner) = unsafe { swift_map.inner.as_ref() } {
                for (key, value) in &inner.0 {
                    converted_map.insert(key.clone(), value.clone());
                }
            }
            converted_map
        };

        Some(Self {
            address,
            message,
            log,
            metadata,
        })
    }
}

#[repr(C)]
pub struct ProblemReportMetadata {
    inner: *mut Map,
}

struct Map(BTreeMap<String, String>);

impl Map {
    fn new() -> Self {
        Map(BTreeMap::new())
    }

    /// Add key and value pair to the map
    ///
    /// # Safety
    ///
    /// - `key` must be a null-terminated UTF-8 string, containing LF-separated machines.
    /// - `value` must be a valid pointer to some valid and aligned pointer-sized memory.
    unsafe fn add(&mut self, key: *const c_char, value: *const c_char) -> bool {
        assert!(
            !key.is_null(),
            "key must not be null (violates safety contract)"
        );
        assert!(
            !value.is_null(),
            "value must not be null (violates safety contract)"
        );

        // SAFETY: See notes above
        let (key, value) = unsafe { (CStr::from_ptr(key), CStr::from_ptr(value)) };

        match key.to_str() {
            Ok(key_str) => match value.to_str() {
                Ok(value_str) => {
                    self.0.insert(key_str.to_owned(), value_str.to_owned());
                    true
                }
                Err(err) => {
                    log::error!("{err:?}");
                    false
                }
            },
            Err(err) => {
                log::error!("{err:?}");
                false
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn swift_problem_report_metadata_new() -> ProblemReportMetadata {
    let map = Box::new(Map::new());
    ProblemReportMetadata {
        inner: Box::into_raw(map),
    }
}

/// Add key and value pair to the `ProblemReportMetadata`
///
/// # Safety
///
/// `map.inner` must be non-null and point to a valid
/// - `key` must be a null-terminated UTF-8 string, containing LF-separated machines.
/// - `value` must be a valid pointer to some valid and aligned pointer-sized memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn swift_problem_report_metadata_add(
    map: ProblemReportMetadata,
    key: *const c_char,
    value: *const c_char,
) -> bool {
    // Safety: We are assuming that `map.inner` is not null and that it is properly initialized.
    if let Some(inner) = unsafe { map.inner.as_mut() } {
        // Safety: We assume that the `inner` object is valid and mutable. The `add` method is
        // safe to call because we know `inner` is a mutable reference to the underlying data.
        unsafe { inner.add(key, value) }
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn swift_problem_report_metadata_free(map: ProblemReportMetadata) {
    if !map.inner.is_null() {
        // SAFETY: `map.inner` must be properly aligned and non-null
        // The caller must guarantee that `map.inner` is not null and has not been freed
        unsafe {
            drop(Box::from_raw(map.inner));
        }
    }
}
