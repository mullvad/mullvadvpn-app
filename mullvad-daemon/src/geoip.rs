use futures::{self, Future};
use mullvad_rpc;
use mullvad_types::location::{AmIMullvad, GeoIpLocation};
use serde_json;


const URI_V4: &str = "https://ipv4.am.i.mullvad.net/json";
const URI_V6: &str = "https://ipv6.am.i.mullvad.net/json";

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Unable to send request to HTTP client.
    #[error(display = "Unable to send GeoIP request to HTTP client")]
    SendRequestError,

    /// The request was dropped without any response
    #[error(display = "The GeoIP request was dropped without any response")]
    NoResponse,

    /// Error in the HTTP client when requesting GeoIP
    #[error(display = "Failed to request GeoIP")]
    Transport(#[error(source)] mullvad_rpc::rest::Error),

    /// Failed to deserialize GeoIP response
    #[error(display = "Failed to deserialize GeoIP response")]
    Deserialize(#[error(source)] serde_json::error::Error),
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
        .map_err(|_| Error::SendRequestError)
        .and_then(|_| response_rx.map_err(|_| Error::NoResponse))
        .and_then(|response_result| response_result.map_err(Error::Transport))
        .and_then(|response| serde_json::from_slice(&response).map_err(Error::Deserialize))
}
