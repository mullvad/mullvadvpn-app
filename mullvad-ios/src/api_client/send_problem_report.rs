use mullvad_api::{
    rest::{self, MullvadRestHandle},
    ProblemReportProxy,
};
use talpid_future::retry::retry_future;

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
    let empty_metadata: BTreeMap<String, String> = BTreeMap::new();

    let future_factory = || api.porblem_report_response(&problem_report_request.address, &problem_report_request.message, &(String::from_utf8_lossy(&problem_report_request.log)), &empty_metadata);

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
}

struct ProblemReportRequest {
    address: String,
    message: String,
    log: Vec<u8>,
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

        Some(Self { address, message, log })
    }
}