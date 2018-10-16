extern crate hyper;

use std::net::IpAddr;
use std::path::Path;

use self::hyper::Uri;
use futures::{self, Future};
use serde_json;

use mullvad_rpc;
use mullvad_types::location::GeoIpLocation;
use talpid_core::cached_dns_resolver::CachedDnsResolver;


const AM_I_MULLVAD_CACHE_FILE: &str = "am-i-mullvad-ip-address.txt";
const AM_I_MULLVAD_HOST: &str = "am.i.mullvad.net";
const AM_I_MULLVAD_IP: [u8; 4] = [46, 166, 184, 225];

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


pub struct GeoLocationFetcher {
    dns_resolver: CachedDnsResolver,
}

impl GeoLocationFetcher {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        let cache_file = cache_dir.as_ref().join(AM_I_MULLVAD_CACHE_FILE);
        let fallback_ip = IpAddr::from(AM_I_MULLVAD_IP);

        GeoLocationFetcher {
            dns_resolver: CachedDnsResolver::new(AM_I_MULLVAD_HOST, cache_file, fallback_ip),
        }
    }

    pub fn send_location_request(
        &mut self,
        request_sender: mullvad_rpc::rest::RequestSender,
    ) -> impl Future<Item = GeoIpLocation, Error = Error> {
        let (response_tx, response_rx) = futures::sync::oneshot::channel();
        let request = mullvad_rpc::rest::create_get_request(self.build_uri());

        futures::Sink::send(request_sender.clone(), (request, response_tx))
            .map_err(|e| Error::with_chain(e, ErrorKind::NoResponse))
            .and_then(|_| response_rx.map_err(|e| Error::with_chain(e, ErrorKind::NoResponse)))
            .and_then(|response_result| response_result.map_err(Error::from))
            .and_then(|response| serde_json::from_slice(&response).map_err(Error::from))
    }

    fn build_uri(&mut self) -> Uri {
        format!("https://{}/json", self.dns_resolver.resolve())
            .parse()
            .unwrap()
    }
}
