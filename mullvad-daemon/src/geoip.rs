extern crate hyper;

use std::path::Path;

use self::hyper::{header, Uri};
use futures::{self, Future};
use serde_json;
use tokio_core::reactor::Handle;

use mullvad_rpc::{
    self,
    uris::{AM_I_MULLVAD_HOST, AM_I_MULLVAD_IP, AM_I_MULLVAD_IP_CACHE_FILE},
    CachedDnsResolver,
};
use mullvad_types::location::GeoIpLocation;


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
    https_handle: mullvad_rpc::rest::RequestSender,
}

impl GeoLocationFetcher {
    pub fn new(
        cache_dir: impl AsRef<Path>,
        ca_path: impl AsRef<Path>,
        handle: &Handle,
    ) -> Result<Self> {
        let cache_file = cache_dir.as_ref().join(AM_I_MULLVAD_IP_CACHE_FILE);

        Ok(GeoLocationFetcher {
            dns_resolver: CachedDnsResolver::new(AM_I_MULLVAD_HOST, cache_file, *AM_I_MULLVAD_IP),
            https_handle: mullvad_rpc::rest::create_https_client_with_sni(
                ca_path,
                Some(AM_I_MULLVAD_HOST.to_owned()),
                handle,
            )?,
        })
    }

    pub fn send_location_request(&mut self) -> impl Future<Item = GeoIpLocation, Error = Error> {
        let (response_tx, response_rx) = futures::sync::oneshot::channel();
        let mut request = mullvad_rpc::rest::create_get_request(self.build_uri());

        request
            .headers_mut()
            .set(header::Host::new(AM_I_MULLVAD_HOST, None));

        futures::Sink::send(self.https_handle.clone(), (request, response_tx))
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
