use futures::channel::oneshot;
use hyper::{Client, Uri};
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::net::SocketAddr;
use tokio_rustls::rustls::ClientConfig;

use crate::{AmIMullvad, Error};

const LE_ROOT_CERT: &[u8] = include_bytes!("../../../mullvad-api/le_root_cert.pem");

static CLIENT_CONFIG: Lazy<ClientConfig> = Lazy::new(|| {
    ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(read_cert_store())
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

pub async fn geoip_lookup(mullvad_host: String) -> Result<AmIMullvad, Error> {
    let uri = Uri::try_from(format!("https://ipv4.am.i.{mullvad_host}/json"))
        .map_err(|_| Error::InvalidUrl)?;
    http_get(uri).await
}

pub async fn http_get<T: DeserializeOwned>(url: Uri) -> Result<T, Error> {
    log::debug!("GET {url}");

    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(CLIENT_CONFIG.clone())
        .https_only()
        .enable_http1()
        .build();

    let client: Client<_, hyper::Body> = Client::builder().build(https);
    let body = client
        .get(url)
        .await
        .map_err(|error| Error::HttpRequest(error.to_string()))?
        .into_body();

    // TODO: limit length
    let bytes = hyper::body::to_bytes(body).await.map_err(|error| {
        log::error!("Failed to convert body to bytes buffer: {}", error);
        Error::DeserializeBody
    })?;

    serde_json::from_slice(&bytes).map_err(|error| {
        log::error!("Failed to deserialize response: {}", error);
        Error::DeserializeBody
    })
}

fn read_cert_store() -> tokio_rustls::rustls::RootCertStore {
    let mut cert_store = tokio_rustls::rustls::RootCertStore::empty();

    let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(LE_ROOT_CERT))
        .expect("Failed to parse pem file");
    let (num_certs_added, num_failures) = cert_store.add_parsable_certificates(&certs);
    if num_failures > 0 || num_certs_added != 1 {
        panic!("Failed to add root cert");
    }

    cert_store
}
