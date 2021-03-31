use futures::join;
use mullvad_rpc::{self, rest::{RequestServiceHandle, Error}};
use mullvad_types::location::{AmIMullvad, GeoIpLocation};
use talpid_types::ErrorExt;

const URI_V4: &str = "https://ipv4.am.i.mullvad.net/json";
const URI_V6: &str = "https://ipv6.am.i.mullvad.net/json";

pub async fn send_location_request(
    request_sender: RequestServiceHandle,
) -> Result<GeoIpLocation, Error> {
    let v4_sender = request_sender.clone();
    let v4_future = async move {
        let location = send_location_request_internal(URI_V4, v4_sender).await?;
        Ok::<GeoIpLocation, Error>(GeoIpLocation::from(location))
    };
    let v6_sender = request_sender.clone();
    let v6_future = async move {
        let location = send_location_request_internal(URI_V6, v6_sender).await?;
        Ok::<GeoIpLocation, Error>(GeoIpLocation::from(location))
    };

    let (v4_result, v6_result) = join!(v4_future, v6_future);

    match (v4_result, v6_result) {
        (Ok(mut v4), Ok(v6)) => {
            v4.ipv6 = v6.ipv6;
            v4.mullvad_exit_ip = v4.mullvad_exit_ip && v6.mullvad_exit_ip;
            Ok(v4)
        }
        (Ok(v4), Err(e)) => {
            log::debug!(
                "{}",
                e.display_chain_with_msg("Unable to fetch IPv6 GeoIP location")
            );
            Ok(v4)
        }
        (Err(e), Ok(v6)) => {
            log::debug!(
                "{}",
                e.display_chain_with_msg("Unable to fetch IPv4 GeoIP location")
            );
            Ok(v6)
        }
        (Err(e_v4), Err(_)) => Err(e_v4),
    }
}

async fn send_location_request_internal(
    uri: &'static str,
    service: RequestServiceHandle,
) -> Result<AmIMullvad, Error> {
    let future_service = service.clone();
    let request = mullvad_rpc::rest::RestRequest::get(uri)?;
    let response = future_service.request(request).await?;
    mullvad_rpc::rest::deserialize_body(response).await
}
