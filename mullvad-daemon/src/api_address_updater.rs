//! A small updater that keeps the API IP address cache up to date by fetching changes from the
//! Mullvad API.
use mullvad_api::{rest::MullvadRestHandle, AddressCache, ApiProxy};
use std::time::Duration;

const API_IP_CHECK_INITIAL: Duration = Duration::from_secs(15 * 60);
const API_IP_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const API_IP_CHECK_ERROR_INTERVAL: Duration = Duration::from_secs(15 * 60);

pub async fn run_api_address_fetcher(address_cache: AddressCache, handle: MullvadRestHandle) {
    #[cfg(feature = "api-override")]
    if API.disable_address_cache {
        return futures::future::pending().await;
    }

    let availability = handle.availability.clone();
    let api_proxy = ApiProxy::new(handle);
    let mut next_delay = API_IP_CHECK_INITIAL;

    loop {
        talpid_time::sleep(next_delay).await;

        if let Err(error) = availability.wait_background().await {
            log::error!("Failed while waiting for API: {}", error);
            continue;
        }
        match api_proxy.clone().get_api_addrs().await {
            Ok(new_addrs) => {
                if let Some(addr) = new_addrs.first() {
                    log::debug!(
                        "Fetched new API address {:?}. Fetching again in {} hours",
                        addr,
                        API_IP_CHECK_INTERVAL.as_secs() / (60 * 60)
                    );
                    if let Err(err) = address_cache.set_address(*addr).await {
                        log::error!("Failed to save newly updated API address: {}", err);
                    }
                } else {
                    log::error!("API returned no API addresses");
                }

                next_delay = API_IP_CHECK_INTERVAL;
            }
            Err(err) => {
                log::error!(
                    "Failed to fetch new API addresses: {}. Retrying in {} seconds",
                    err,
                    API_IP_CHECK_ERROR_INTERVAL.as_secs()
                );

                next_delay = API_IP_CHECK_ERROR_INTERVAL;
            }
        }
    }
}
