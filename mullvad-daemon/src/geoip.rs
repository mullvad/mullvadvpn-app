use futures::join;
use mullvad_api::{
    self,
    rest::{Error, RequestServiceHandle},
};
use mullvad_types::location::{AmIMullvad, GeoIpLocation};
use talpid_types::ErrorExt;

const LOCATION_HOST_URI: &str = "am.i.mullvad.net";

#[cfg(feature = "api-override")]
lazy_static::lazy_static! {
    static ref MULLVAD_LOCATION_HOST: Option<String> = std::env::var("MULLVAD_LOCATION_HOST")
        .ok();
}

pub async fn send_location_request(
    request_sender: RequestServiceHandle,
    use_ipv6: bool,
) -> Result<GeoIpLocation, Error> {
    let v4_sender = request_sender.clone();
    let v4_future = async move {
        #[cfg(not(feature = "api-override"))]
        let uri = LOCATION_HOST_URI;
        #[cfg(feature = "api-override")]
        let uri = MULLVAD_LOCATION_HOST
            .as_deref()
            .unwrap_or(LOCATION_HOST_URI);

        let uri_v4 = format!("https://ipv4.{}/json", uri);
        let location = send_location_request_internal(&uri_v4, v4_sender).await?;
        Ok::<GeoIpLocation, Error>(GeoIpLocation::from(location))
    };
    let v6_sender = request_sender.clone();
    let v6_future = async move {
        if use_ipv6 {
            #[cfg(not(feature = "api-override"))]
            let uri = LOCATION_HOST_URI;
            #[cfg(feature = "api-override")]
            let uri = MULLVAD_LOCATION_HOST
                .as_deref()
                .unwrap_or(LOCATION_HOST_URI);

            let uri_v6 = format!("https://ipv6.{}/json", uri);
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
    let request = mullvad_api::rest::RestRequest::get(uri)?;
    let response = future_service.request(request).await?;
    mullvad_api::rest::deserialize_body(response).await
}

fn log_network_error(err: Error, version: &'static str) {
    let err_message = &format!("Unable to fetch {version} GeoIP location");
    match err {
        Error::HyperError(hyper_err) if hyper_err.is_connect() => {
            if let Some(cause) = hyper_err.into_cause() {
                if let Some(err) = cause.downcast_ref::<std::io::Error>() {
                    // Don't log ENETUNREACH errors, they are not informative.
                    if err.raw_os_error() == Some(libc::ENETUNREACH) {
                        return;
                    }
                    log::debug!("{}: Hyper connect error: {}", err_message, cause);
                }
            } else {
                log::error!("Hyper Connection error did not contain a cause!");
            }
        }
        any_other_error => {
            log::debug!("{}", any_other_error.display_chain_with_msg(err_message));
        }
    };
}
