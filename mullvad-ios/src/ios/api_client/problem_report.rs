use mullvad_api::ProblemReportProxy;
use std::{collections::HashMap, sync::Arc};

use crate::{
    api_client::{ApiContext, ApiContextError},
    type_bridges::TokioBridge,
};

use super::{do_request_with_empty_body, response::ApiResponse, retry_strategy::RetryStrategy};

#[uniffi::export]
impl ApiContext {
    /// Send a problem report via the Mullvad API client.
    pub async fn send_problem_report(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        problem_report_request: ProblemReportRequest,
    ) -> Result<ApiResponse, ApiContextError> {
        let api = ProblemReportProxy::new(self.rest_handle());

        let metadata = problem_report_request.metadata.into_iter().collect();
        let future_factory = move || {
            api.problem_report(
                &problem_report_request.address,
                &problem_report_request.message,
                &problem_report_request.log,
                &metadata,
            )
        };

        TokioBridge::spawn(do_request_with_empty_body(*retry_strategy, future_factory))
            .await
            .map_err(ApiContextError::from)
    }
}

#[derive(uniffi::Record)]
pub struct ProblemReportRequest {
    address: String,
    message: String,
    log: String,
    metadata: HashMap<String, String>,
}
