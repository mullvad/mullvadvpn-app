use super::{
    cancellation::RequestCancelHandle, do_request, response::ApiResponse, retry_request,
    retry_strategy::RetryStrategy,
};
use crate::api_client::{ApiContext, access_method_settings::AccessMethodSettingWrapper};
use chrono::{DateTime, Utc};
use mullvad_api::{
    ApiProxy, RelayListProxy,
    relay_list_transparency::{RelayListDigest, SigsumPayload},
    rest::{self, MullvadRestHandle},
};
use std::sync::Arc;

#[uniffi::export]
impl ApiContext {
    pub fn get_addresses(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
    ) -> Arc<RequestCancelHandle> {
        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| match get_addresses_inner(
                api_context.rest_handle(),
                retry_strategy,
            )
            .await
            {
                Ok(response) => completion_handler.finish(response),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(ApiResponse::rest_error(err));
                }
            },
        )
    }

    pub fn api_addrs_available(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        access_method_setting: Arc<AccessMethodSettingWrapper>,
    ) -> Arc<RequestCancelHandle> {
        let access_method_setting = access_method_setting.inner.clone();

        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| match api_context
                .access_mode_handler
                .resolve(access_method_setting.clone())
                .await
            {
                Ok(Some(resolved_connection_mode)) => {
                    let oneshot_client = api_context
                        .api_client
                        .mullvad_rest_handle(resolved_connection_mode.connection_mode);

                    match api_addrs_available_inner(oneshot_client, retry_strategy).await {
                        Ok(_) => completion_handler.finish(ApiResponse::ok()),
                        Err(err) => {
                            log::error!("{err:?}");
                            completion_handler.finish(ApiResponse::rest_error(err));
                        }
                    }
                }
                Ok(None) => {
                    log::error!("Invalid access method configuration, {access_method_setting:?}");
                    completion_handler.finish(ApiResponse::access_method_error(
                        mullvad_api::access_mode::Error::Resolve {
                            access_method: access_method_setting.access_method,
                        },
                    ));
                }
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(ApiResponse::access_method_error(err));
                }
            },
        )
    }

    pub fn get_relays(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        digest: Option<String>,
        digest_timestamp: Option<i64>,
    ) -> Arc<RequestCancelHandle> {
        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| {
                let digest = if let Some(digest) = digest {
                    match RelayListDigest::try_from(digest) {
                        Ok(digest) => Some(digest),
                        Err(err) => {
                            log::error!("bad relay digest: {err:?}");
                            completion_handler.finish(ApiResponse::cancelled());
                            return;
                        }
                    }
                } else {
                    None
                };

                match get_relays_inner(
                    api_context.rest_handle(),
                    retry_strategy,
                    digest,
                    digest_timestamp.and_then(DateTime::from_timestamp_millis),
                )
                .await
                {
                    Ok(response) => completion_handler.finish(response),
                    Err(err) => {
                        log::error!("{err:?}");
                        completion_handler.finish(ApiResponse::rest_error(err));
                    }
                }
            },
        )
    }
}

async fn get_addresses_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<ApiResponse, rest::Error> {
    let api = ApiProxy::new(rest_client);

    let future_factory = || api.get_api_addrs_response();

    do_request(retry_strategy, future_factory).await
}

async fn get_relays_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    digest: Option<RelayListDigest>,
    digest_timestamp: Option<DateTime<Utc>>,
) -> Result<ApiResponse, rest::Error> {
    let api = RelayListProxy::new(rest_client);

    let sigsum_payload = digest
        .zip(digest_timestamp)
        .map(|(digest, timestamp)| SigsumPayload::new(digest, timestamp));

    let future_factory = || api.relay_list_response(sigsum_payload.clone());

    let response = retry_request(retry_strategy, future_factory).await?;
    ApiResponse::with_sigsum_verified_body(response)
}

async fn api_addrs_available_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<bool, rest::Error> {
    let api = ApiProxy::new(rest_client);

    let future_factory = || api.api_addrs_available();
    retry_request(retry_strategy, future_factory).await
}
