use crate::{
    api_client::{ApiContext, retry_request, retry_strategy::RetryStrategy},
    type_bridges::{
        erased_error::ErasedError,
        ip_addr::{Ipv4Addr, Ipv6Addr},
    },
};
use mullvad_api::DefaultDnsResolver;
use mullvad_types::location::{AmIMullvad, GeoIpLocation};
use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct IAmMullvadResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub exit_ip: bool,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
}

#[uniffi::export]
impl ApiContext {
    pub async fn am_i_mullvad(
        self: Arc<Self>,
        use_ipv6: bool,
        hostname: String,
        retry_strategy: Arc<RetryStrategy>,
    ) -> Result<IAmMullvadResponse, ErasedError> {
        let runtime = crate::mullvad_ios_runtime().unwrap();
        let context = self.clone();
        let response = runtime
            .spawn(
                async move { request_inner(&context, use_ipv6, &hostname, *retry_strategy).await },
            )
            .await??;
        Ok(IAmMullvadResponse {
            latitude: response.latitude,
            longitude: response.longitude,
            exit_ip: response.mullvad_exit_ip,
            ipv4: response.ipv4.map(Into::into),
            ipv6: response.ipv6.map(Into::into),
        })
    }
}

async fn request_inner(
    context: &ApiContext,
    ipv6: bool,
    hostname: &str,
    retry_strategy: RetryStrategy,
) -> Result<GeoIpLocation, ErasedError> {
    let rest_handle = context.api_client.rest_handle(DefaultDnsResolver);

    let factory = || async {
        let request = mullvad_api::rest::get(&format!(
            "https://{}.{hostname}/json",
            if ipv6 { "ipv6" } else { "ipv4" }
        ))?;
        rest_handle.request(request).await
    };
    retry_request(retry_strategy, factory)
        .await?
        .deserialize::<AmIMullvad>()
        .await
        .map(GeoIpLocation::from)
        .map_err(ErasedError::from)
}
