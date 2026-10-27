use std::sync::Arc;

use mullvad_api::AccountsProxy;

use crate::{
    api_client::{ApiContext, ApiContextError},
    type_bridges::TokioBridge,
};

use super::{do_request, response::ApiResponse, retry_strategy::RetryStrategy};

#[uniffi::export]
impl ApiContext {
    pub async fn init_storekit_payment(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
    ) -> Result<ApiResponse, ApiContextError> {
        TokioBridge::spawn(async move {
            let account_proxy = AccountsProxy::new(self.rest_handle());
            let future_factory = || account_proxy.init_storekit_payment(account_number.clone());
            do_request(*retry_strategy, future_factory).await
        })
        .await
        .map_err(ApiContextError::from)
    }

    pub async fn check_storekit_payment(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        body: Vec<u8>,
    ) -> Result<ApiResponse, ApiContextError> {
        let account_proxy = AccountsProxy::new(self.rest_handle());
        let future_factory = || account_proxy.check_storekit_payment(body.clone());

        do_request(*retry_strategy, future_factory)
            .await
            .map_err(ApiContextError::from)
    }
}
