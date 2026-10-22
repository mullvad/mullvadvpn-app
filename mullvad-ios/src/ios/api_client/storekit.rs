use std::sync::Arc;

use mullvad_api::{
    AccountsProxy,
    rest::{self, MullvadRestHandle},
};

use crate::{
    api_client::ApiContext,
    type_bridges::{AnyError, TokioBridge},
};

use super::{
    cancellation::RequestCancelHandle, do_request, response::ApiResponse,
    retry_strategy::RetryStrategy,
};

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

async fn check_storekit_payment_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    body: Vec<u8>,
) -> Result<ApiResponse, rest::Error> {
    let account_proxy = AccountsProxy::new(rest_client);

    let future_factory = || account_proxy.check_storekit_payment(body.clone());

    do_request(retry_strategy, future_factory).await
}
