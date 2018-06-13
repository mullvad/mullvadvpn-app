use futures::{self, Future};
use mullvad_rpc;
use mullvad_types::location::GeoIpLocation;
use serde_json;


const URI: &str = "https://am.i.mullvad.net/json";

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
) -> Box<Future<Item = GeoIpLocation, Error = Error>> {
    let (response_tx, response_rx) = futures::sync::oneshot::channel();
    let request = mullvad_rpc::rest::create_get_request(URI.parse().unwrap());
    let future = futures::Sink::send(request_sender.clone(), (request, response_tx))
        .map_err(|e| Error::with_chain(e, ErrorKind::NoResponse))
        .and_then(|_| response_rx.map_err(|e| Error::with_chain(e, ErrorKind::NoResponse)))
        .and_then(|response_result| response_result.map_err(Error::from))
        .and_then(|response| serde_json::from_slice(&response).map_err(Error::from));
    Box::new(future)
}
