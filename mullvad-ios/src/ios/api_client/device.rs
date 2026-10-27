use super::{
    do_request, do_request_with_empty_body, response::ApiResponse, retry_strategy::RetryStrategy,
};
use crate::{
    api_client::{ApiContext, ApiContextError},
    type_bridges::TokioBridge,
};
use mullvad_api::DevicesProxy;
use std::sync::Arc;
use talpid_types::net::wireguard::PublicKey;

#[uniffi::export]
impl ApiContext {
    /// Get device info via the Mullvad API client.
    pub async fn get_device(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
        identifier: String,
    ) -> Result<ApiResponse, ApiContextError> {
        let api = DevicesProxy::new(self.rest_handle());
        let future_factory = move || api.get_response(account_number.clone(), identifier.clone());

        TokioBridge::spawn(do_request(*retry_strategy, future_factory))
            .await
            .map_err(ApiContextError::from)
    }

    /// Get devices info via the Mullvad API client.
    pub async fn get_devices(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
    ) -> Result<ApiResponse, ApiContextError> {
        let rest_client = self.rest_handle();
        let api = DevicesProxy::new(rest_client);
        let future_factory = move || api.list_response(account_number.clone());

        TokioBridge::spawn(do_request(*retry_strategy, future_factory))
            .await
            .map_err(ApiContextError::from)
    }

    /// create device via the Mullvad API client.
    pub async fn create_device(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
        public_key: &[u8],
    ) -> Result<ApiResponse, ApiContextError> {
        let Ok(pub_key): Result<[u8; 32], _> = public_key.try_into() else {
            return Ok(ApiResponse::bad_public_key_size());
        };

        let pub_key = PublicKey::from(pub_key);
        let api = DevicesProxy::new(self.rest_handle());
        let future_factory = move || api.create_response(account_number.clone(), pub_key.clone());

        TokioBridge::spawn(do_request(*retry_strategy, future_factory))
            .await
            .map_err(ApiContextError::from)
    }

    /// Delete device via the Mullvad API client.
    pub async fn delete_device(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
        identifier: String,
    ) -> Result<ApiResponse, ApiContextError> {
        let rest_client = self.rest_handle();

        let api = DevicesProxy::new(rest_client);
        let future_factory = move || api.remove(account_number.clone(), identifier.clone());

        TokioBridge::spawn(do_request_with_empty_body(*retry_strategy, future_factory))
            .await
            .map_err(ApiContextError::from)
    }

    /// Rotate device key via the Mullvad API client.
    pub async fn rotate_device_key(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
        identifier: String,
        public_key: &[u8],
    ) -> Result<ApiResponse, ApiContextError> {
        let Ok(pub_key): Result<[u8; 32], _> = public_key.try_into() else {
            return Ok(ApiResponse::bad_public_key_size());
        };

        let pub_key = PublicKey::from(pub_key);
        let api = DevicesProxy::new(self.rest_handle());
        let future_factory = move || {
            api.replace_wg_key_response(account_number.clone(), identifier.clone(), pub_key.clone())
        };

        TokioBridge::spawn(do_request(*retry_strategy, future_factory))
            .await
            .map_err(ApiContextError::from)
    }
}
