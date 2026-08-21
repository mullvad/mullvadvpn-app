use mullvad_api::{
    ProblemReportProxy,
    rest::{self, MullvadRestHandle},
};
use std::{collections::HashMap, sync::Arc};

use crate::api_client::ApiContext;

use super::{
    cancellation::RequestCancelHandle, do_request_with_empty_body,
    response::SwiftMullvadApiResponse, retry_strategy::RetryStrategy,
};

/// Send a problem report via the Mullvad API client.
#[uniffi::export]
pub fn mullvad_ios_send_problem_report(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    request: ProblemReportRequest,
) -> Arc<RequestCancelHandle> {
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
                Ok(response) => completion_handler.finish(Arc::new(response)),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(Arc::new(SwiftMullvadApiResponse::rest_error(err)));
                }
            }
        },
    )
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
