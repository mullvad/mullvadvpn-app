use std::sync::Arc;

use mullvad_api::AccountsProxy;

use crate::{
    api_client::ApiContext,
    type_bridges::{AnyError, TokioBridge},
};

use super::{do_request, response::ApiResponse, retry_strategy::RetryStrategy};

#[uniffi::export]
impl ApiContext {
    pub async fn init_storekit_payment(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
    ) -> Result<ApiResponse, AnyError> {
        TokioBridge::spawn(async move {
            let account_proxy = AccountsProxy::new(self.rest_handle());
            let future_factory = || account_proxy.init_storekit_payment(account_number.clone());
            do_request(*retry_strategy, future_factory).await
        })
        .await
        .map_err(AnyError::from)
        .inspect_err(|e| log::error!("init_storekit_payment failed: {e}"))
    }

    pub async fn check_storekit_payment(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        body: Vec<u8>,
    ) -> Result<ApiResponse, AnyError> {
        let account_proxy = AccountsProxy::new(self.rest_handle());
        let future_factory = || account_proxy.check_storekit_payment(body.clone());

        do_request(*retry_strategy, future_factory)
            .await
            .map_err(AnyError::from)
            .inspect_err(|e| log::error!("check_storekit_payment failed: {e}"))
    }
}
