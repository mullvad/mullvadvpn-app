use futures::{self, Future};
use mullvad_rpc;
use mullvad_types::location::{AmIMullvad, GeoIpLocation};
use serde_json;


const URI_V4: &str = "https://ipv4.am.i.mullvad.net/json";
const URI_V6: &str = "https://ipv6.am.i.mullvad.net/json";

error_chain! {
    errors {
        NoResponse { description("The request was dropped without any response") }
    }
    links {
        Transport(mullvad_rpc::rest::Error, mullvad_rpc::rest::ErrorKind);
    }
    foreign_links {
        Deserialize(serde_json::error::Error);
    }
}


pub fn send_location_request(
    request_sender: mullvad_rpc::rest::RequestSender,
) -> impl Future<Item = GeoIpLocation, Error = Error> {
    let v4_future =
        send_location_request_internal(URI_V4, request_sender.clone()).map(GeoIpLocation::from);
    let v6_future = send_location_request_internal(URI_V6, request_sender).map(GeoIpLocation::from);

    v4_future.then(|v4_result| {
        v6_future.then(|v6_result| match (v4_result, v6_result) {
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
        })
    })
}

fn send_location_request_internal(
    uri: &'static str,
    request_sender: mullvad_rpc::rest::RequestSender,
) -> impl Future<Item = AmIMullvad, Error = Error> {
    let (response_tx, response_rx) = futures::sync::oneshot::channel();
    let request = mullvad_rpc::rest::create_get_request(uri.parse().unwrap());

    futures::Sink::send(request_sender, (request, response_tx))
        .map_err(|e| Error::with_chain(e, ErrorKind::NoResponse))
        .and_then(|_| response_rx.map_err(|e| Error::with_chain(e, ErrorKind::NoResponse)))
        .and_then(|response_result| response_result.map_err(Error::from))
        .and_then(|response| serde_json::from_slice(&response).map_err(Error::from))
}
