use std::sync::Arc;

use mullvad_api::{
    AccountsProxy,
    rest::{self, MullvadRestHandle},
};

use crate::api_client::ApiContext;

use super::{
    cancellation::RequestCancelHandle, do_request, do_request_with_empty_body,
    response::ApiResponse, retry_strategy::RetryStrategy,
};

#[uniffi::export]
pub fn mullvad_ios_get_account(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    account_number: String,
) -> Arc<RequestCancelHandle> {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion| match mullvad_ios_get_account_inner(
            api_context.rest_handle(),
            retry_strategy,
            account_number,
        )
        .await
        {
            Ok(response) => completion.finish(Arc::new(response)),
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(Arc::new(ApiResponse::rest_error(err)));
            }
        },
    )
}

#[uniffi::export]
pub fn mullvad_ios_create_account(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
) -> Arc<RequestCancelHandle> {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_create_account_inner(api_context.rest_handle(), retry_strategy).await
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

#[uniffi::export]
pub fn mullvad_ios_delete_account(
    api_context: Arc<ApiContext>,
    retry_strategy: Arc<RetryStrategy>,
    account_number: String,
) -> Arc<RequestCancelHandle> {
    RequestCancelHandle::new(
        api_context,
        retry_strategy,
        async move |api_context, retry_strategy, completion_handler| {
            match mullvad_ios_delete_account_inner(
                api_context.rest_handle(),
                retry_strategy,
                account_number,
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

async fn mullvad_ios_get_account_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
) -> Result<ApiResponse, rest::Error> {
    let api = AccountsProxy::new(rest_client);
    let future_factory = || api.get_data_response(account_number.clone());

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_create_account_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<ApiResponse, rest::Error> {
    let api = AccountsProxy::new(rest_client);
    let future_factory = || api.create_account_response();

    do_request(retry_strategy, future_factory).await
}

async fn mullvad_ios_delete_account_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: String,
) -> Result<ApiResponse, rest::Error> {
    let api = AccountsProxy::new(rest_client);
    let future_factory = || api.delete_account(account_number.clone());

    do_request_with_empty_body(retry_strategy, future_factory).await
}
