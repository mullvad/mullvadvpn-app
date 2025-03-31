use mullvad_api::{
    rest::{self, MullvadRestHandle},
    ProblemReportProxy,
};
use std::ffi::CStr;
use std::os::raw::c_char;
use talpid_future::retry::retry_future;

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    completion::{CompletionCookie, SwiftCompletionHandler},
    response::SwiftMullvadApiResponse,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
    SwiftApiContext,
};

use mullvad_api::rest::Error;
use std::collections::BTreeMap;
use tokio::task::JoinHandle;

#[no_mangle]
pub unsafe extern "C" fn mullvad_api_send_problem_report(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    request: SwiftProblemReportRequest,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));
    let completion = completion_handler.clone();

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let problem_report_request = match unsafe {
        ProblemReportRequest::from_swift_parameters(request)
    } {
        Some(req) => req,
        None => {
            let err = Error::ApiError(
                rest::StatusCode::BAD_REQUEST,
                "Failed to send problem report: invalid address, message, or log data.".to_string(),
            );
            log::error!("{err:?}");
            completion.finish(SwiftMullvadApiResponse::rest_error(err));
            return SwiftCancelHandle::empty();
        }
    };

    let task: JoinHandle<()> = tokio_handle.spawn(async move {
        match do_request(
            api_context.rest_handle(),
            retry_strategy,
            problem_report_request,
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

async fn do_request(
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
            &problem_report_request.meta_data,
        )
    };

    let should_retry = |result: &Result<_, rest::Error>| match result {
        Err(err) => err.is_network_error(),
        Ok(_) => false,
    };

    retry_future(future_factory, should_retry, retry_strategy.delays()).await?;
    SwiftMullvadApiResponse::ok()
}

#[repr(C)]
pub struct SwiftProblemReportRequest {
    address: *const u8,
    message: *const u8,
    log: *const u8,
    meta_data: ProblemReportMetadata,
}

struct ProblemReportRequest {
    address: String,
    message: String,
    log: Vec<u8>,
    meta_data: BTreeMap<String, String>,
}

unsafe impl Send for SwiftProblemReportRequest {}

impl ProblemReportRequest {
    unsafe fn from_swift_parameters(request: SwiftProblemReportRequest) -> Option<Self> {
        fn get_string(ptr: *const u8) -> String {
            if ptr.is_null() {
                return String::new();
            }

            unsafe {
                CStr::from_ptr(ptr.cast())
                    .to_str()
                    .map(ToOwned::to_owned)
                    .unwrap_or_default()
            }
        }

        let address = get_string(request.address);
        let message = get_string(request.message);
        let log = get_string(request.log).into();

        let meta_data = if request.meta_data.inner.is_null() {
            BTreeMap::new()
        } else {
            let swift_map = &request.meta_data;
            let mut converted_map = BTreeMap::new();

            if let Some(inner) = swift_map.inner.as_ref() {
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
            meta_data,
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

    unsafe fn add(&mut self, key: *const c_char, value: *const c_char) -> bool {
        if key.is_null() || value.is_null() {
            log::error!("Failed to add metadata: key or value is NULL.");
            return false;
        }
        let key = unsafe { CStr::from_ptr(key) };
        let value = unsafe { CStr::from_ptr(value) };

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

#[no_mangle]
pub extern "C" fn swift_problem_report_meta_data_new() -> ProblemReportMetadata {
    let map = Box::new(Map::new());
    ProblemReportMetadata {
        inner: Box::into_raw(map),
    }
}

#[no_mangle]
pub extern "C" fn swift_problem_report_meta_data_add(
    map: ProblemReportMetadata,
    key: *const c_char,
    value: *const c_char,
) -> bool {
    if let Some(inner) = unsafe { map.inner.as_mut() } {
        unsafe { inner.add(key, value) }
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn swift_problem_report_meta_data_free(mut map: ProblemReportMetadata) {
    if !map.inner.is_null() {
        unsafe {
            drop(Box::from_raw(map.inner));
            map.inner = std::ptr::null_mut();
        }
    }
}
