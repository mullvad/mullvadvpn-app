use crate::{
    api_client::{ApiContext, retry_request, retry_strategy::RetryStrategy},
    type_bridges::AmIMullvadResponse,
};
use mullvad_api::DefaultDnsResolver;
use mullvad_types::location::AmIMullvad;
use std::sync::Arc;

#[uniffi::export]
impl ApiContext {
    pub async fn am_i_mullvad(
        self: Arc<Self>,
        address: String,
        retry_strategy: Arc<RetryStrategy>,
    ) -> Option<AmIMullvadResponse> {
        let runtime = crate::mullvad_ios_runtime()
            .inspect_err(|e| log::error!("failed to grab tokio runtime: {e}"))
            .ok()?;
        let context = self.clone();
        runtime
            .spawn(async move { request_inner(&context, &address, *retry_strategy).await })
            .await
            .inspect_err(|e| log::error!("tokio task failure: {e}"))
            .ok()?
            .inspect_err(|e| log::error!("rest error: {e}"))
            .ok()
            .map(AmIMullvadResponse::from)
    }
}

async fn request_inner(
    context: &ApiContext,
    address: &str,
    retry_strategy: RetryStrategy,
) -> Result<AmIMullvad, mullvad_api::rest::Error> {
    let rest_handle = context.api_client.rest_handle(DefaultDnsResolver);

    let factory = || async {
        let request = mullvad_api::rest::get(address)?;
        rest_handle.request(request).await
    };
    retry_request(retry_strategy, factory)
        .await?
        .deserialize::<AmIMullvad>()
        .await
}
