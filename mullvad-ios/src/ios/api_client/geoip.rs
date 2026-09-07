use crate::{
    api_client::{ApiContext, retry_request, retry_strategy::RetryStrategy},
    type_bridges::{AmIMullvadResponse, AnyError},
};
use mullvad_api::rest::RequestServiceHandle;
use mullvad_types::location::AmIMullvad;
use std::sync::Arc;

pub struct AmIMullvadHandles {
    pub ipv4: RequestServiceHandle,
    pub ipv6: RequestServiceHandle,
}

#[uniffi::export]
impl ApiContext {
    pub async fn am_i_mullvad(
        self: Arc<Self>,
        ipv6: bool,
        retry_strategy: Arc<RetryStrategy>,
    ) -> Result<AmIMullvadResponse, AnyError> {
        let runtime = crate::mullvad_ios_runtime()
            .inspect_err(|e| log::error!("failed to grab tokio runtime: {e}"))
            .map_err(AnyError::message)?;
        let context = self.clone();
        runtime
            .spawn(async move { request_inner(&context, ipv6, *retry_strategy).await })
            .await
            .inspect_err(|e| log::error!("tokio task failure: {e}"))?
            .inspect_err(|e| log::error!("rest error: {e}"))
            .map(AmIMullvadResponse::from)
            .map_err(AnyError::from)
    }
}

async fn request_inner(
    context: &ApiContext,
    ipv6: bool,
    retry_strategy: RetryStrategy,
) -> Result<AmIMullvad, mullvad_api::rest::Error> {
    let rest_handle = if ipv6 {
        context.am_i_handle.ipv6.clone()
    } else {
        context.am_i_handle.ipv4.clone()
    };

    let factory = async || rest_handle.request().get("json")?.await;
    let ok = retry_request(retry_strategy, factory)
        .await?
        .deserialize::<AmIMullvad>()
        .await?;
    Ok(ok)
}
