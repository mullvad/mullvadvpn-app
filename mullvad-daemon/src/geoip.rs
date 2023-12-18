use std::time::Duration;

use futures::join;
use mullvad_api::{
    self,
    availability::ApiAvailabilityHandle,
    rest::{Error, RequestServiceHandle},
};
use mullvad_types::location::{AmIMullvad, GeoIpLocation};
use once_cell::sync::Lazy;
use talpid_core::future_retry::{retry_future, ExponentialBackoff, Jittered};
use talpid_types::ErrorExt;

// Define the Mullvad connection checking api endpoint.
//
// In a development build the host name for the connection checking endpoint can
// be overriden by defining the env variable `MULLVAD_CONNCHECK_HOST`.
//
// If `MULLVAD_CONNCHECK_HOST` is set when running `mullvad-daemon` in a
// production build, a warning will be logged and the env variable *won´t* have
// any effect on the api call. The default host name `am.i.mullvad.net` will
// always be used in release mode.
static MULLVAD_CONNCHECK_HOST: Lazy<String> = Lazy::new(|| {
    const DEFAULT_CONNCHECK_HOST: &str = "am.i.mullvad.net";
    let conncheck_host_var = std::env::var("MULLVAD_CONNCHECK_HOST").ok();
    let host = if cfg!(feature = "api-override") {
        match conncheck_host_var.as_deref() {
            Some(host) => {
                log::debug!("Overriding conncheck endpoint. Using {}", &host);
                host
            }
            None => DEFAULT_CONNCHECK_HOST,
        }
    } else {
        if conncheck_host_var.is_some() {
            log::warn!("These variables are ignored in production builds: MULLVAD_CONNCHECK_HOST");
        };
        DEFAULT_CONNCHECK_HOST
    };
    host.to_string()
});

const LOCATION_RETRY_STRATEGY: Jittered<ExponentialBackoff> =
    Jittered::jitter(ExponentialBackoff::new(Duration::from_secs(1), 4));

/// Fetch the current `GeoIpLocation` with retrys
pub async fn get_geo_location(
    rest_service: RequestServiceHandle,
    use_ipv6: bool,
    api_handle: ApiAvailabilityHandle,
) -> Option<GeoIpLocation> {
    log::debug!("Fetching GeoIpLocation");
    match retry_future(
        move || send_location_request(rest_service.clone(), use_ipv6),
        move |result| match result {
            Err(error) if error.is_network_error() => !api_handle.get_state().is_offline(),
            _ => false,
        },
        LOCATION_RETRY_STRATEGY,
    )
    .await
    {
        Ok(loc) => Some(loc),
        Err(e) => {
            log::warn!("Unable to fetch GeoIP location: {}", e.display_chain());
            None
        }
    }
}

async fn send_location_request(
    request_sender: RequestServiceHandle,
    use_ipv6: bool,
) -> Result<GeoIpLocation, Error> {
    let v4_sender = request_sender.clone();
    let v4_future = async move {
        let uri_v4 = format!("https://ipv4.{}/json", *MULLVAD_CONNCHECK_HOST);
        let location = send_location_request_internal(&uri_v4, v4_sender).await?;
        Ok::<GeoIpLocation, Error>(GeoIpLocation::from(location))
    };
    let v6_sender = request_sender.clone();
    let v6_future = async move {
        if use_ipv6 {
            let uri_v6 = format!("https://ipv6.{}/json", *MULLVAD_CONNCHECK_HOST);
            let location = send_location_request_internal(&uri_v6, v6_sender).await;
            Some(location.map(GeoIpLocation::from))
        } else {
            None
        }
    };

    let (v4_result, v6_result) = join!(v4_future, v6_future);

    match (v4_result, v6_result) {
        (Ok(mut v4), Some(Ok(v6))) => {
            v4.ipv6 = v6.ipv6;
            v4.mullvad_exit_ip = v4.mullvad_exit_ip && v6.mullvad_exit_ip;
            Ok(v4)
        }
        (Ok(v4), None) => Ok(v4),
        (Ok(v4), Some(Err(e))) => {
            log_network_error(e, "IPv6");
            Ok(v4)
        }
        (Err(e), Some(Ok(v6))) => {
            log_network_error(e, "IPv4");
            Ok(v6)
        }
        (Err(e_v4), _) => Err(e_v4),
    }
}

async fn send_location_request_internal(
    uri: &str,
    service: RequestServiceHandle,
) -> Result<AmIMullvad, Error> {
    let future_service = service.clone();
    let request = mullvad_api::rest::Request::get(uri)?;
    future_service.request(request).await?.deserialize().await
}

fn log_network_error(err: Error, version: &'static str) {
    if !err.is_offline() {
        let err_message = &format!("Unable to fetch {version} GeoIP location");
        log::debug!("{}", err.display_chain_with_msg(err_message));
    }
}
