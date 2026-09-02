use std::sync::Arc;

use mullvad_api::{
    AccountsProxy,
    rest::{self, MullvadRestHandle},
};
use mullvad_types::account::AccountNumber;

use crate::api_client::ApiContext;

use super::{
    cancellation::RequestCancelHandle, do_request, response::ApiResponse,
    retry_strategy::RetryStrategy,
};

#[uniffi::export]
impl ApiContext {
    pub fn init_storekit_payment(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        account_number: String,
    ) -> Arc<RequestCancelHandle> {
        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| {
                match init_storekit_payment_inner(
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

    pub fn check_storekit_payment(
        self: Arc<Self>,
        retry_strategy: Arc<RetryStrategy>,
        body: Vec<u8>,
    ) -> Arc<RequestCancelHandle> {
        RequestCancelHandle::new(
            self,
            retry_strategy,
            async move |api_context, retry_strategy, completion_handler| {
                match check_storekit_payment_inner(api_context.rest_handle(), retry_strategy, body)
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
}

async fn init_storekit_payment_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account_number: AccountNumber,
) -> Result<ApiResponse, rest::Error> {
    let account_proxy = AccountsProxy::new(rest_client);

    let future_factory = || account_proxy.init_storekit_payment(account_number.clone());

    do_request(retry_strategy, future_factory).await
}

async fn check_storekit_payment_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    body: Vec<u8>,
) -> Result<ApiResponse, rest::Error> {
    let account_proxy = AccountsProxy::new(rest_client);

    let future_factory = || account_proxy.check_storekit_payment(body.clone());

    do_request(retry_strategy, future_factory).await
}
