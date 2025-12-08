use crate::{AmIMullvad, Error};
use bytes::Bytes;
use futures::channel::oneshot;
use http_body_util::{BodyExt, Full};
use hyper::Uri;
use hyper_util::client::legacy::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio_rustls::rustls::{
    self, ClientConfig,
    pki_types::{CertificateDer, pem::PemObject},
};

const LE_ROOT_CERT: &[u8] = include_bytes!("../../../mullvad-api/le_root_cert.pem");

static CLIENT_CONFIG: LazyLock<ClientConfig> = LazyLock::new(|| {
    ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(read_cert_store().expect("Failed to parse pem file"))
        .with_no_client_auth()
});

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SockHandleId(pub usize);

pub struct SockHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    bind_addr: SocketAddr,
}

impl SockHandle {
    pub(crate) async fn start_tcp_forward(
        client: crate::service::ServiceClient,
        bind_addr: SocketAddr,
        via_addr: SocketAddr,
    ) -> Result<Self, Error> {
        let (stop_tx, stop_rx) = oneshot::channel();

        let (id, bind_addr) = client
            .start_tcp_forward(tarpc::context::current(), bind_addr, via_addr)
            .await??;

        tokio::spawn(async move {
            let _ = stop_rx.await;

            log::trace!("Stopping TCP forward");

            if let Err(error) = client.stop_tcp_forward(tarpc::context::current(), id).await {
                log::error!("Failed to stop TCP forward: {error}");
            }
        });

        Ok(SockHandle {
            stop_tx: Some(stop_tx),
            bind_addr,
        })
    }

    pub fn stop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
    }

    pub fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }
}

impl Drop for SockHandle {
    fn drop(&mut self) {
        self.stop()
    }
}

pub async fn geoip_lookup(mullvad_host: String, timeout: Duration) -> Result<AmIMullvad, Error> {
    let uri = Uri::try_from(format!("https://ipv4.am.i.{mullvad_host}/json"))
        .map_err(|_| Error::InvalidUrl)?;
    http_get_with_timeout(uri, timeout).await
}

pub async fn http_get<T: DeserializeOwned>(url: Uri) -> Result<T, Error> {
    log::debug!("GET {url}");

    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(CLIENT_CONFIG.clone())
        .https_only()
        .enable_http1()
        .build();

    let client: Client<_, Full<Bytes>> =
        Client::builder(hyper_util::rt::TokioExecutor::new()).build(https);
    let body = client
        .get(url)
        .await
        .map_err(|error| Error::HttpRequest(error.to_string()))?
        .into_body();

    // TODO: limit length
    let bytes = body
        .collect()
        .await
        .map_err(|error| {
            log::error!("Failed to collect response body: {}", error);
            Error::DeserializeBody
        })?
        .to_bytes();

    serde_json::from_slice(&bytes).map_err(|error| {
        log::error!("Failed to deserialize response: {}", error);
        Error::DeserializeBody
    })
}

pub async fn http_get_with_timeout<T: DeserializeOwned>(
    url: Uri,
    timeout: Duration,
) -> Result<T, Error> {
    tokio::time::timeout(timeout, http_get(url))
        .await
        .map_err(|_| Error::HttpRequest("Request timed out".into()))?
}

fn read_cert_store() -> Result<rustls::RootCertStore, rustls_pki_types::pem::Error> {
    let mut cert_store = rustls::RootCertStore::empty();

    let certs = CertificateDer::pem_reader_iter(&mut std::io::BufReader::new(LE_ROOT_CERT))
        .collect::<Result<Vec<_>, _>>()?;
    let (num_certs_added, num_failures) = cert_store.add_parsable_certificates(certs);
    if num_failures > 0 || num_certs_added != 1 {
        panic!("Failed to add root cert");
    }

    Ok(cert_store)
}
