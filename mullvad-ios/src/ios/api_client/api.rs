use std::sync::Arc;

use mullvad_api::{
    ApiProxy, ETag, RelayListProxy,
    rest::{self, MullvadRestHandle},
};

use crate::api_client::{ApiContext, access_method_settings::AccessMethodSettingWrapper};

use super::{
    cancellation::RequestCancelHandle, do_request, response::ApiResponse,
    retry_request, retry_strategy::RetryStrategy,
};

#[uniffi::export]
pub fn mullvad_ios_get_addresses(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
) -> Arc<RequestCancelHandle> {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_get_addresses_inner(api_context.rest_handle(), retry_strategy).await {
                Ok(response) => completion_handler.finish(Arc::new(response)),
                Err(err) => {
                    log::error!("{err:?}");
                    completion_handler.finish(Arc::new(ApiResponse::rest_error(err)));
                }
            }
        },
    )
}

#[uniffi::export]
pub fn mullvad_ios_api_addrs_available(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    access_method_setting: Arc<AccessMethodSettingWrapper>,
) -> Arc<RequestCancelHandle> {
    let access_method_setting = access_method_setting.inner.clone();

    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| match api_context
            .access_mode_handler
            .resolve(access_method_setting.clone())
            .await
        {
            Ok(Some(resolved_connection_mode)) => {
                let oneshot_client = api_context
                    .api_client
                    .mullvad_rest_handle(resolved_connection_mode.connection_mode.into_provider());

                match mullvad_ios_api_addrs_available_inner(oneshot_client, retry_strategy).await {
                    Ok(_) => completion_handler.finish(Arc::new(ApiResponse::ok())),
                    Err(err) => {
                        log::error!("{err:?}");
                        completion_handler
                            .finish(Arc::new(ApiResponse::rest_error(err)));
                    }
                }
            }
            Ok(None) => {
                log::error!("Invalid access method configuration, {access_method_setting:?}");
                completion_handler.finish(Arc::new(ApiResponse::access_method_error(
                    mullvad_api::access_mode::Error::Resolve {
                        access_method: access_method_setting.access_method,
                    },
                )));
            }
            Err(err) => {
                log::error!("{err:?}");
                completion_handler
                    .finish(Arc::new(ApiResponse::access_method_error(err)));
            }
        },
    )
}

#[uniffi::export]
pub fn mullvad_ios_get_relays(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    etag: Option<String>,
) -> Arc<RequestCancelHandle> {
    let maybe_etag: Option<ETag> = etag.map(ETag);

    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_get_relays_inner(
                api_context.rest_handle(),
                retry_strategy,
                maybe_etag,
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

async fn mullvad_ios_get_addresses_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<ApiResponse, rest::Error> {
    let api = ApiProxy::new(rest_client);

    let future_factory = || api.get_api_addrs_response();

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_get_relays_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    etag: Option<ETag>,
) -> Result<ApiResponse, rest::Error> {
    let api = RelayListProxy::new(rest_client);

    let future_factory = || api.relay_list_response(etag.clone());

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_api_addrs_available_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<bool, rest::Error> {
    let api = ApiProxy::new(rest_client);

    let future_factory = || api.api_addrs_available();
    retry_request(retry_strategy, future_factory).await
}
