use mullvad_api::{
    rest::{self, MullvadRestHandle},
    ProblemReportProxy,
};
use talpid_future::retry::retry_future;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    completion::{CompletionCookie, SwiftCompletionHandler},
    response::SwiftMullvadApiResponse,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
    SwiftApiContext,
};

use mullvad_api::rest::Error;
use std::{collections::BTreeMap};
use std::slice;
use tokio::task::JoinHandle;

#[no_mangle]
pub unsafe extern "C" fn mullvad_api_send_problem_report(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    request: *const SwiftProblemReportRequest,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));
    let completion = completion_handler.clone();

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let problem_report_request = match unsafe { ProblemReportRequest::from_swift_parameters(request) } {
        Some(req) => req,
        None => {
            let err = Error::ApiError(rest::StatusCode::BAD_REQUEST, "Failed to send problem report: invalid address, message, or log data.".to_string());
            log::error!("{err:?}");
            completion.finish(SwiftMullvadApiResponse::rest_error(err));
            return SwiftCancelHandle::empty();
        }
    };

    let task: JoinHandle<()> = tokio_handle.spawn(async move {
        match mullvad_api_send_problem_report_inner(
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

async fn mullvad_api_send_problem_report_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    problem_report_request: ProblemReportRequest,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = ProblemReportProxy::new(rest_client);

    let future_factory = || api.porblem_report_response(&problem_report_request.address, &problem_report_request.message, &(String::from_utf8_lossy(&problem_report_request.log)), &problem_report_request.meta_data);

    let should_retry = |result: &Result<_, rest::Error>| match result {
        Err(err) => err.is_network_error(),
        Ok(_) => false,
    };

    let response = retry_future(future_factory, should_retry, retry_strategy.delays()).await?;
    SwiftMullvadApiResponse::with_body(response).await

}

#[repr(C)]
pub struct SwiftProblemReportRequest {
    address: *const u8,
    address_len: usize,
    message: *const u8,
    message_len: usize,
    log: *const u8,
    log_len: usize,
    meta_data : *mut SwiftMap,
}

struct ProblemReportRequest {
    address: String,
    message: String,
    log: Vec<u8>,
    meta_data: BTreeMap<String, String>,
}


unsafe impl Send for SwiftProblemReportRequest {}

impl ProblemReportRequest {
    unsafe fn from_swift_parameters(request: *const SwiftProblemReportRequest) -> Option<Self> {
        if request.is_null() {
            return None;
        }

        let request = &*request; // Dereference the pointer

        let address_slice = slice::from_raw_parts(request.address, request.address_len);
        let message_slice = slice::from_raw_parts(request.message, request.message_len);
        let log_slice = slice::from_raw_parts(request.log, request.log_len);

        let address = String::from_utf8(address_slice.to_vec()).ok()?;
        let message = String::from_utf8(message_slice.to_vec()).ok()?;
        let log = log_slice.to_vec();

        let meta_data = if request.meta_data.is_null() {
            BTreeMap::new() // Default empty map
        } else {
            let swift_map = &*request.meta_data;
            let mut converted_map = BTreeMap::new();

            if let Some(inner) = swift_map.inner.as_ref() {
                for (key, value) in &inner.0 {
                    converted_map.insert(key.clone(), value.clone());
                }
            }

            converted_map
        };

        Some(Self { address, message, log, meta_data })
    }
}




#[repr(C)]
pub struct SwiftMap {
    inner: *mut Map,
}

struct Map(BTreeMap<String, String>);

impl Map {
    fn new() -> Self {
        Map(BTreeMap::new())
    }

    unsafe fn add(&mut self, key: *const c_char, value: *const c_char) {
        let key = CStr::from_ptr(key).to_string_lossy().into_owned();
        let value = CStr::from_ptr(value).to_string_lossy().into_owned();
        self.0.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }
}

#[no_mangle]
pub extern "C" fn swift_map_new() -> *mut SwiftMap {
    let map = Box::new(Map::new());
    let swift_map = Box::new(SwiftMap {
        inner: Box::into_raw(map),
    });
    Box::into_raw(swift_map)
}

#[no_mangle]
pub extern "C" fn swift_map_add(map: *mut SwiftMap, key: *const c_char, value: *const c_char) {
    if let Some(map) = unsafe { map.as_mut() } {
        if let Some(inner) = unsafe { map.inner.as_mut() } {
            unsafe { inner.add(key, value) };
        }
    }
}

#[no_mangle]
pub extern "C" fn swift_map_get(map: *mut SwiftMap, key: *const c_char) -> *const c_char {
    if let Some(map) = unsafe { map.as_mut() } {
        if let Some(inner) = unsafe { map.inner.as_ref() } {
            let key = unsafe { CStr::from_ptr(key).to_str().unwrap() };
            if let Some(value) = inner.get(key) {
                return CString::new(value).unwrap().into_raw();
            }
        }
    }
    std::ptr::null()
}

#[no_mangle]
pub extern "C" fn swift_map_free(map: *mut SwiftMap) {
    if map.is_null() {
        return;
    }

    unsafe {
        // Free the inner map first
        if let Some(inner) = (*map).inner.as_mut() {
            drop(Box::from_raw(inner));
        }

        // Free the SwiftMap struct itself
        drop(Box::from_raw(map));
    }
}
