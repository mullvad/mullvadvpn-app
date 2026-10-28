use super::{
    do_request, do_request_with_empty_body, response::ApiResponse, retry_strategy::RetryStrategy,
};
use crate::{
    api_client::ApiContext,
    type_bridges::{AnyError, TokioBridge},
};
use mullvad_api::AccountsProxy;
use std::sync::Arc;

#[uniffi::export]
impl ApiContext {
    pub async fn get_account(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
    ) -> Result<ApiResponse, AnyError> {
        let api = AccountsProxy::new(self.rest_handle());
        let future_factory = move || api.get_data_response(account_number.clone());

        TokioBridge::spawn(do_request(*retry_strategy, future_factory))
            .await
            .map_err(AnyError::from)
    }

    pub async fn create_account(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
    ) -> Result<ApiResponse, AnyError> {
        let api = AccountsProxy::new(self.rest_handle());
        let future_factory = move || api.create_account_response();

        TokioBridge::spawn(do_request(*retry_strategy, future_factory))
            .await
            .map_err(AnyError::from)
    }

    pub async fn delete_account(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
    ) -> Result<ApiResponse, AnyError> {
        let api = AccountsProxy::new(self.rest_handle());
        let future_factory = move || api.delete_account(account_number.clone());

        TokioBridge::spawn(do_request_with_empty_body(*retry_strategy, future_factory))
            .await
            .map_err(AnyError::from)
    }
}
