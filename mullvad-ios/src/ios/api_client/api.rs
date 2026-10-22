use super::{do_request, response::ApiResponse, retry_request, retry_strategy::RetryStrategy};
use crate::{
    api_client::{ApiContext, access_method_settings::AccessMethodSettingWrapper},
    type_bridges::{AnyError, TokioBridge},
};
use chrono::DateTime;
use mullvad_api::{
    ApiProxy, RelayListProxy,
    relay_list_transparency::{RelayListDigest, SigsumPayload},
};
use std::sync::Arc;

#[uniffi::export]
impl ApiContext {
    pub async fn get_addresses(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
    ) -> Result<ApiResponse, AnyError> {
        TokioBridge::spawn(async move {
            let api = Arc::new(ApiProxy::new(self.rest_handle()));

            let future_factory = || api.get_api_addrs_response();
            do_request(*retry_strategy, future_factory).await
        })
        .await
        .map_err(AnyError::from)
        .inspect_err(|e| log::error!("get_addresses failed: {e}"))
    }

    pub async fn api_addrs_available(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        access_method_setting: Arc<AccessMethodSettingWrapper>,
    ) -> Result<ApiResponse, AnyError> {
        TokioBridge::spawn(async move {
            let access_method_setting = access_method_setting.inner.clone();

            let resolved_connection_mode = self
                .access_mode_handler
                .resolve(access_method_setting.clone())
                .await?;
            let Some(resolved_connection_mode) = resolved_connection_mode else {
                log::error!("Invalid access method configuration, {access_method_setting:?}");
                return Err(AnyError::from(mullvad_api::access_mode::Error::Resolve {
                    access_method: access_method_setting.access_method,
                }));
            };

            let oneshot_client = self
                .api_client
                .mullvad_rest_handle(resolved_connection_mode.connection_mode);

            let api = ApiProxy::new(oneshot_client);

            let future_factory = || api.api_addrs_available();

            retry_request(*retry_strategy, future_factory)
                .await
                .map(|_| ApiResponse::ok())
                .map_err(AnyError::from)
        })
        .await
        .inspect_err(|e| log::error!("api_addrs_available failed: {e}"))
    }

    pub async fn get_relays(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        digest: Option<String>,
        digest_timestamp: Option<i64>,
    ) -> Result<ApiResponse, AnyError> {
        let digest = if let Some(digest) = digest {
            match RelayListDigest::try_from(digest) {
                Ok(digest) => Some(digest),
                Err(err) => {
                    log::error!("bad relay digest: {err:?}");
                    return Ok(ApiResponse::cancelled());
                }
            }
        } else {
            None
        };

        let digest_timestamp = digest_timestamp.and_then(DateTime::from_timestamp_millis);

        TokioBridge::spawn(async move {
            let api = RelayListProxy::new(self.rest_handle());

            let sigsum_payload = digest
                .zip(digest_timestamp)
                .map(|(digest, timestamp)| SigsumPayload::new(digest, timestamp));

            let future_factory = || api.relay_list_response(sigsum_payload.clone());

            let response = retry_request(*retry_strategy, future_factory).await?;
            ApiResponse::with_sigsum_verified_body(response)
        })
        .await
        .map_err(AnyError::from)
        .inspect_err(|e| log::error!("get_addresses failed: {e}"))
    }
}
