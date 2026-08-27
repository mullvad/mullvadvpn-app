use mullvad_api::{
    ProblemReportProxy,
    rest::{self, MullvadRestHandle},
};
use std::{collections::HashMap, sync::Arc};

use crate::api_client::ApiContext;

use super::{
    cancellation::RequestCancelHandle, do_request_with_empty_body, response::ApiResponse,
    retry_strategy::RetryStrategy,
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
                    completion_handler.finish(Arc::new(ApiResponse::rest_error(err)));
                }
            }
        },
    )
}

async fn mullvad_ios_send_problem_report_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    problem_report_request: ProblemReportRequest,
) -> Result<ApiResponse, rest::Error> {
    let api = ProblemReportProxy::new(rest_client);

    let metadata = problem_report_request.metadata.into_iter().collect();
    let future_factory = || {
        api.problem_report(
            &problem_report_request.address,
            &problem_report_request.message,
            &problem_report_request.log,
            &metadata,
        )
    };

    do_request_with_empty_body(retry_strategy, future_factory).await
}

#[derive(uniffi::Record)]
pub struct ProblemReportRequest {
    address: String,
    message: String,
    log: String,
    metadata: HashMap<String, String>,
}
