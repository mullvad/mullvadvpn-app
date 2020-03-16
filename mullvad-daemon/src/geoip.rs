use futures::{self, Future};
use mullvad_rpc::{self, rest::RequestServiceHandle};
use mullvad_types::location::{AmIMullvad, GeoIpLocation};


const URI_V4: &str = "https://ipv4.am.i.mullvad.net/json";
const URI_V6: &str = "https://ipv6.am.i.mullvad.net/json";

pub fn send_location_request(
    request_sender: RequestServiceHandle,
) -> impl Future<Item = GeoIpLocation, Error = mullvad_rpc::rest::Error> {
    let v4_future =
        send_location_request_internal(URI_V4, request_sender.clone()).map(GeoIpLocation::from);
    let v6_future = send_location_request_internal(URI_V6, request_sender).map(GeoIpLocation::from);

    v4_future.then(
        |v4_result: Result<GeoIpLocation, mullvad_rpc::rest::Error>| {
            v6_future.then(
                |v6_result: Result<GeoIpLocation, mullvad_rpc::rest::Error>| match (
                    v4_result, v6_result,
                ) {
                    (Ok(mut v4), Ok(v6)) => {
                        v4.ipv6 = v6.ipv6;
                        v4.mullvad_exit_ip = v4.mullvad_exit_ip && v6.mullvad_exit_ip;
                        Ok(v4)
                    }
                    (Ok(v4), Err(e)) => {
                        log::debug!("Unable to fetch IPv6 GeoIP location: {}", e);
                        Ok(v4)
                    }
                    (Err(e), Ok(v6)) => {
                        log::debug!("Unable to fetch IPv4 GeoIP location: {}", e);
                        Ok(v6)
                    }
                    (Err(e_v4), Err(_)) => Err(e_v4),
                },
            )
        },
    )
}

fn send_location_request_internal(
    uri: &'static str,
    service: RequestServiceHandle,
) -> impl Future<Item = AmIMullvad, Error = mullvad_rpc::rest::Error> {
    let future_service = service.clone();
    let future = async move {
        let request = mullvad_rpc::rest::RestRequest::get(uri)?;
        let response = future_service.request(request).await?;
        mullvad_rpc::rest::deserialize_body(response).await
    };
    service.compat_spawn(future)
}
